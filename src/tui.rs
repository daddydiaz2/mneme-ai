/// Comprehensive TUI for mneme-ai — ecosystem configurator.
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Row, Table, Tabs},
    Frame, Terminal,
};
use std::io;

use crate::mneme;
use crate::profile::{ModelAssignment, ProfileStore, SddProfile, SDD_PHASES};
use crate::skills;

type Term = Terminal<ratatui::backend::CrosstermBackend<io::Stdout>>;

pub fn run_tui() -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let store = ProfileStore::new();
    store.init()?;
    let _ = skills::install_mneme_skills();

    let mut app = App::new(store);
    let res = app.run(&mut terminal);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    res
}

#[derive(Clone, Copy, PartialEq)]
enum Screen {
    Welcome,
    AgentInstall,
    Profiles,
    ProfileDetail(usize),
    CreateProfile,
    Skills,
    SkillsDetail(usize),
    Memory,
    Backups,
    Help,
}

const TAB_TITLES: &[&str] = &[
    "🏠 Welcome",
    "💻 Agents",
    "⚙ Profiles",
    "📚 Skills",
    "🧠 Memory",
    "💾 Backups",
    "❓ Help",
];

struct App {
    store: ProfileStore,
    profiles: Vec<SddProfile>,
    skills_list: Vec<skills::Skill>,
    selected: usize,
    screen: Screen,
    tab_index: usize,
    status: String,
    // Form state
    form_name: String,
    form_provider: String,
    form_model: String,
    form_step: usize,
    // Install state
    install_step: usize,
    install_log: Vec<String>,
}

impl App {
    fn new(store: ProfileStore) -> Self {
        let profiles = store.list().unwrap_or_default();
        let skill_list = skills::scan_skills();
        Self {
            store,
            profiles,
            skills_list: skill_list,
            selected: 0,
            screen: Screen::Welcome,
            tab_index: 0,
            status: String::new(),
            form_name: String::new(),
            form_provider: String::new(),
            form_model: String::new(),
            form_step: 0,
            install_step: 0,
            install_log: Vec::new(),
        }
    }

    fn run(&mut self, terminal: &mut Term) -> anyhow::Result<()> {
        loop {
            terminal.draw(|f| self.render(f))?;
            if let Event::Key(key) = event::read()? {
                if !self.handle_key(key.code) {
                    break;
                }
            }
        }
        Ok(())
    }

    fn handle_key(&mut self, key: KeyCode) -> bool {
        match self.screen {
            Screen::Welcome => self.handle_welcome(key),
            Screen::AgentInstall => self.handle_agent_install(key),
            Screen::Profiles => self.handle_profiles(key),
            Screen::ProfileDetail(_) => match key {
                KeyCode::Esc | KeyCode::Backspace => {
                    self.screen = Screen::Profiles;
                    self.tab_index = 2;
                    true
                }
                KeyCode::Char('q') => false,
                _ => true,
            },
            Screen::CreateProfile => self.handle_create_profile(key),
            Screen::Skills => self.handle_skills(key),
            Screen::SkillsDetail(_) => match key {
                KeyCode::Esc => {
                    self.screen = Screen::Skills;
                    self.tab_index = 3;
                    true
                }
                KeyCode::Char('q') => false,
                _ => true,
            },
            Screen::Memory => match key {
                KeyCode::Esc | KeyCode::Backspace => {
                    self.screen = Screen::Welcome;
                    self.tab_index = 0;
                    true
                }
                KeyCode::Char('q') => false,
                _ => true,
            },
            Screen::Backups => match key {
                KeyCode::Esc | KeyCode::Backspace => {
                    self.screen = Screen::Welcome;
                    self.tab_index = 0;
                    true
                }
                KeyCode::Char('q') => false,
                _ => true,
            },
            Screen::Help => match key {
                KeyCode::Esc | KeyCode::Backspace => {
                    self.screen = Screen::Welcome;
                    self.tab_index = 0;
                    true
                }
                KeyCode::Char('q') => false,
                _ => true,
            },
        }
    }

    fn handle_welcome(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Char('1') => {
                self.screen = Screen::AgentInstall;
                self.tab_index = 1;
                self.install_step = 0;
                self.install_log.clear();
                true
            }
            KeyCode::Char('2') => {
                self.screen = Screen::Profiles;
                self.tab_index = 2;
                true
            }
            KeyCode::Char('3') => {
                self.screen = Screen::Skills;
                self.tab_index = 3;
                true
            }
            KeyCode::Char('4') => {
                self.screen = Screen::Memory;
                self.tab_index = 4;
                true
            }
            KeyCode::Char('5') => {
                self.screen = Screen::Backups;
                self.tab_index = 5;
                true
            }
            KeyCode::Char('6') => {
                self.screen = Screen::Help;
                self.tab_index = 6;
                true
            }
            KeyCode::Char('q') | KeyCode::Esc => false,
            _ => true,
        }
    }

    fn handle_agent_install(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Tab | KeyCode::Enter => {
                match self.install_step {
                    0 => { self.install_step = 1; }
                    1 => {
                        self.install_log.push("✓ Step 1: OpenCode selected".to_string());
                        self.install_step = 2;
                    }
                    2 => {
                        self.install_log.push("  Creating mneme-orchestrator...".to_string());
                        let _ = crate::install::install_agent("opencode");
                        let _ = crate::install::install_opencode_agents();
                        self.install_step = 3;
                    }
                    3 => {
                        self.install_log.push("  Installing skills + branding...".to_string());
                        let _ = skills::install_mneme_skills();
                        let _ = crate::opencode::customize_opencode();
                        self.skills_list = skills::scan_skills();
                        self.install_step = 4;
                    }
                    4 => {
                        self.install_log.push("✅ Setup complete!".to_string());
                        self.install_step = 5;
                    }
                    _ => {}
                }
                true
            }
            KeyCode::Esc | KeyCode::Backspace => {
                if self.install_step > 0 && self.install_step < 5 {
                    self.install_step -= 1;
                } else {
                    self.screen = Screen::Welcome; self.tab_index = 0;
                    self.install_step = 0; self.install_log.clear();
                }
                true
            }
            KeyCode::Char('q') => false,
            _ => true,
        }
    }

    fn handle_profiles(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Up | KeyCode::Char('k') => {
                self.selected = self.selected.saturating_sub(1);
                true
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let max = self.profiles.len().saturating_sub(1);
                self.selected = self.selected.saturating_add(1).min(max);
                true
            }
            KeyCode::Enter => {
                if self.selected < self.profiles.len() {
                    self.screen = Screen::ProfileDetail(self.selected);
                }
                true
            }
            KeyCode::Char('n') => {
                self.screen = Screen::CreateProfile;
                self.form_step = 0;
                self.form_name.clear();
                self.form_provider.clear();
                self.form_model.clear();
                true
            }
            KeyCode::Char('d') => {
                if let Some(p) = self.profiles.get(self.selected).map(|p| p.name.clone()) {
                    if p != "default" {
                        self.store.delete(&p).ok();
                        self.profiles = self.store.list().unwrap_or_default();
                    }
                }
                true
            }
            KeyCode::Esc | KeyCode::Backspace => {
                self.screen = Screen::Welcome;
                self.tab_index = 0;
                true
            }
            KeyCode::Char('q') => false,
            _ => true,
        }
    }

    fn handle_skills(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Up | KeyCode::Char('k') => {
                self.selected = self.selected.saturating_sub(1);
                true
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.selected = self
                    .selected
                    .saturating_add(1)
                    .min(self.skills_list.len().saturating_sub(1));
                true
            }
            KeyCode::Enter => {
                if self.selected < self.skills_list.len() {
                    self.screen = Screen::SkillsDetail(self.selected);
                }
                true
            }
            KeyCode::Char('r') => {
                self.skills_list = skills::scan_skills();
                let _ = skills::write_registry(&self.skills_list);
                self.status = format!("Registry refreshed: {} skills", self.skills_list.len());
                true
            }
            KeyCode::Esc => {
                self.screen = Screen::Welcome;
                self.tab_index = 0;
                true
            }
            KeyCode::Char('q') => false,
            _ => true,
        }
    }

    fn handle_create_profile(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Char(c) if c != '\n' && c != '\t' => {
                match self.form_step {
                    0 => self.form_name.push(c),
                    1 => self.form_provider.push(c),
                    2 => self.form_model.push(c),
                    _ => {}
                }
                true
            }
            KeyCode::Backspace => {
                match self.form_step {
                    0 => {
                        self.form_name.pop();
                    }
                    1 => {
                        self.form_provider.pop();
                    }
                    2 => {
                        self.form_model.pop();
                    }
                    _ => {}
                }
                true
            }
            KeyCode::Tab | KeyCode::Enter => {
                if self.form_step < 2 {
                    self.form_step += 1;
                } else if !self.form_name.is_empty() {
                    self.store
                        .save(&SddProfile {
                            name: self.form_name.clone().to_lowercase().replace(' ', "-"),
                            orchestrator: ModelAssignment {
                                provider: if self.form_provider.is_empty() {
                                    "opencode".to_string()
                                } else {
                                    self.form_provider.clone()
                                },
                                model: if self.form_model.is_empty() {
                                    "default".to_string()
                                } else {
                                    self.form_model.clone()
                                },
                                reasoning_effort: None,
                            },
                            phases: std::collections::HashMap::new(),
                        })
                        .ok();
                    self.profiles = self.store.list().unwrap_or_default();
                    self.status = format!("Created: {}", self.form_name);
                    self.screen = Screen::Profiles;
                }
                true
            }
            KeyCode::Esc => {
                self.screen = Screen::Profiles;
                true
            }
            KeyCode::Char('q') => false,
            _ => true,
        }
    }

    fn render(&self, f: &mut Frame) {
        let area = f.size();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(1)])
            .split(area);

        let tab_titles: Vec<&str> = TAB_TITLES.iter().map(|t| *t).collect();
        let tabs = Tabs::new(tab_titles)
            .select(self.tab_index)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("mneme-ai v0.5.0"),
            )
            .style(Style::default().fg(Color::White))
            .highlight_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            );
        f.render_widget(tabs, chunks[0]);

        match self.screen {
            Screen::Welcome => self.render_welcome(f, chunks[1]),
            Screen::AgentInstall => self.render_install(f, chunks[1]),
            Screen::Profiles => self.render_profiles(f, chunks[1]),
            Screen::ProfileDetail(i) => self.render_profile_detail(f, chunks[1], i),
            Screen::CreateProfile => self.render_create_form(f, chunks[1]),
            Screen::Skills => self.render_skills(f, chunks[1]),
            Screen::SkillsDetail(i) => self.render_skill_detail(f, chunks[1], i),
            Screen::Memory => self.render_memory(f, chunks[1]),
            Screen::Backups => self.render_backups(f, chunks[1]),
            Screen::Help => self.render_help(f, chunks[1]),
        }
    }

    fn render_welcome(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(6), Constraint::Min(3)])
            .split(area);

        // Status dashboard
        let cards = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(34),
            ])
            .split(chunks[0]);

        // Brain status
        let brain_ok = mneme::find_mneme().is_some();
        let guardian_ok = which("mneme-g");
        let brain_info = vec![
            Line::from(Span::styled(
                if brain_ok {
                    "🧠 mneme-brain ✅"
                } else {
                    "🧠 mneme-brain ❌"
                },
                Style::default().fg(if brain_ok { Color::Green } else { Color::Red }),
            )),
            Line::from(if brain_ok {
                "Memory system ready"
            } else {
                "Install: cargo install mneme-brain"
            }),
        ];
        f.render_widget(
            Paragraph::new(brain_info).block(Block::default().borders(Borders::ALL).title("Brain")),
            cards[0],
        );

        // Config status
        let profile_count = self.profiles.len();
        let skill_count = self.skills_list.len();
        let config_info = vec![
            Line::from(format!("SDD Profiles: {}", profile_count)),
            Line::from(format!("Skills: {}", skill_count)),
            Line::from(format!(
                "Agents supported: {}",
                crate::agents::SUPPORTED_AGENTS.len()
            )),
        ];
        f.render_widget(
            Paragraph::new(config_info)
                .block(Block::default().borders(Borders::ALL).title("Config")),
            cards[1],
        );

        // Quick actions
        let actions = vec![
            Line::from(Span::styled(
                "1: Setup Agent  2: Profiles",
                Style::default().fg(Color::Cyan),
            )),
            Line::from(Span::styled(
                "3: Skills  4: Memory  5: Backups  6: Help",
                Style::default().fg(Color::Cyan),
            )),
            Line::from(Span::styled("q: Quit", Style::default().fg(Color::Red))),
        ];
        f.render_widget(
            Paragraph::new(actions).block(Block::default().borders(Borders::ALL).title("Actions")),
            cards[2],
        );

        // Ecosystem status
        let mneme_guardian_found = std::process::Command::new("which")
            .arg("mneme-g")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        let ecosystem = vec![
            Line::from(Span::styled(
                concat!("Ecosystem: ", "mneme-brain ✓ ",),
                Style::default().fg(if brain_ok { Color::Green } else { Color::Red }),
            )),
            Line::from(Span::styled(
                if mneme_guardian_found {
                    "mneme-guardian ✓"
                } else {
                    "mneme-guardian ✗"
                },
                Style::default().fg(if mneme_guardian_found {
                    Color::Green
                } else {
                    Color::Red
                }),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "5: Full setup — configure everything at once",
                Style::default().fg(Color::Yellow),
            )),
        ];
        f.render_widget(
            Paragraph::new(ecosystem)
                .block(Block::default().borders(Borders::ALL).title("Ecosystem")),
            chunks[1],
        );
    }

    fn render_install(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default().direction(Direction::Vertical)
            .constraints([Constraint::Min(1)]).split(area);

        let wizard_steps = [
            "Welcome — configure your ecosystem",
            "Select agent (OpenCode)",
            "Create mneme-orchestrator agents",
            "Install skills + branding",
            "Complete!",
        ];
        
        let mut lines = vec![
            Line::from(Span::styled("🚀 Ecosystem Setup Wizard", Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan))),
            Line::from(""),
        ];

        // Show wizard steps with current highlights
        for (i, step) in wizard_steps.iter().enumerate() {
            let (icon, style) = if i < self.install_step {
                ("✅", Style::default().fg(Color::Green))
            } else if i == self.install_step {
                ("▶ ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            } else {
                ("○ ", Style::default().fg(Color::DarkGray))
            };
            lines.push(Line::from(Span::styled(format!("  {} {}", icon, step), style)));
        }

        lines.push(Line::from(""));
        
        // Show install log
        for log_line in &self.install_log {
            let style = if log_line.starts_with("✓") || log_line.starts_with("✅") {
                Style::default().fg(Color::Green)
            } else if log_line.starts_with("✗") {
                Style::default().fg(Color::Red)
            } else if log_line.starts_with("🚀") {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            lines.push(Line::from(Span::styled(log_line.clone(), style)));
        }

        // Bottom controls
        lines.push(Line::from(""));
        if self.install_step < 5 {
            lines.push(Line::from(Span::styled("  Tab/Enter: Next step  |  Esc: Back  |  q: Quit", Style::default().fg(Color::DarkGray))));
        } else {
            lines.push(Line::from(Span::styled("  Esc: Back to Welcome  |  q: Quit", Style::default().fg(Color::DarkGray))));
        }

        f.render_widget(Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title("💻 Install")), chunks[0]);
    }

    fn render_profiles(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3), Constraint::Length(3)])
            .split(area);

        let items: Vec<ListItem> = self
            .profiles
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let style = if i == self.selected {
                    Style::default().bg(Color::Blue).fg(Color::White)
                } else {
                    Style::default()
                };
                let phases = if p.phases.is_empty() {
                    "default".to_string()
                } else {
                    format!("{} phases", p.phases.len())
                };
                ListItem::new(format!(
                    " {}  [{}/{}]  {}",
                    p.name, p.orchestrator.provider, p.orchestrator.model, phases
                ))
                .style(style)
            })
            .collect();

        f.render_widget(
            List::new(items)
                .block(Block::default().borders(Borders::ALL).title("⚙ Profiles"))
                .highlight_style(Style::default().add_modifier(Modifier::BOLD)),
            chunks[0],
        );

        let msg = if !self.status.is_empty() {
            format!("  |  {}", self.status)
        } else {
            String::new()
        };
        f.render_widget(
            Paragraph::new(format!(
                "↑↓/jk Navigate  Enter View  n Create  d Delete  Esc Back  q Quit{}",
                msg
            ))
            .block(Block::default().borders(Borders::ALL)),
            chunks[1],
        );
    }

    fn render_profile_detail(&self, f: &mut Frame, area: Rect, idx: usize) {
        let p = &self.profiles[idx];
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(1),
                Constraint::Length(3),
            ])
            .split(area);

        f.render_widget(
            Paragraph::new(format!(
                "📋 {}  |  {}/{}",
                p.name, p.orchestrator.provider, p.orchestrator.model
            ))
            .style(Style::default().add_modifier(Modifier::BOLD))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Profile Detail"),
            ),
            chunks[0],
        );

        let mut rows = vec![
            Row::new(vec!["Phase", "Provider", "Model", "Source"])
                .style(Style::default().add_modifier(Modifier::BOLD)),
            Row::new(vec![
                "🔄 orchestrator",
                p.orchestrator.provider.as_str(),
                p.orchestrator.model.as_str(),
                "profile",
            ])
            .style(Style::default().fg(Color::Cyan)),
        ];
        for phase in SDD_PHASES {
            let has_override = p.phases.contains_key(*phase);
            let a = p.phases.get(*phase).unwrap_or(&p.orchestrator);
            let src = if has_override {
                "override"
            } else {
                "inherited"
            };
            let style = if has_override {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            rows.push(
                Row::new(vec![phase, a.provider.as_str(), a.model.as_str(), src]).style(style),
            );
        }
        f.render_widget(
            Table::new(
                rows,
                [
                    Constraint::Length(20),
                    Constraint::Length(15),
                    Constraint::Length(20),
                    Constraint::Length(12),
                ],
            )
            .block(Block::default().borders(Borders::ALL)),
            chunks[1],
        );

        f.render_widget(
            Paragraph::new("Esc: Back").block(Block::default().borders(Borders::ALL)),
            chunks[2],
        );
    }

    fn render_create_form(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(1),
                Constraint::Length(3),
            ])
            .split(area);

        f.render_widget(
            Paragraph::new("Create New SDD Profile")
                .style(Style::default().add_modifier(Modifier::BOLD))
                .block(Block::default().borders(Borders::ALL)),
            chunks[0],
        );

        let indicator = |s: usize| if self.form_step == s { "◄" } else { "✓" };
        let fields = vec![
            Line::from(format!(
                "Profile name [a-z, hyphens]: {} {}",
                self.form_name,
                indicator(0)
            )),
            Line::from(format!(
                "Provider [default: opencode]: {} {}",
                self.form_provider,
                indicator(1)
            )),
            Line::from(format!(
                "Model [default: default]: {} {}",
                self.form_model,
                indicator(2)
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Tab/Enter: Next  |  Esc: Cancel",
                Style::default().fg(Color::Cyan),
            )),
        ];
        f.render_widget(
            Paragraph::new(fields).block(Block::default().borders(Borders::ALL)),
            chunks[1],
        );
    }

    fn render_skills(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(3),
                Constraint::Length(3),
            ])
            .split(area);

        let stats = skills::skill_stats();
        let header =
            Paragraph::new(stats).block(Block::default().borders(Borders::ALL).title("📚 Skills"));
        f.render_widget(header, chunks[0]);

        let items: Vec<ListItem> = self
            .skills_list
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let style = if i == self.selected {
                    Style::default().bg(Color::Blue).fg(Color::White)
                } else {
                    Style::default()
                };
                let st = match s.skill_type {
                    skills::SkillType::Sdd => "sdd",
                    skills::SkillType::Review => "review",
                    skills::SkillType::Judgment => "judge",
                    skills::SkillType::Workflow => "workflow",
                    skills::SkillType::Other => "other",
                };
                ListItem::new(format!(" [{}] {}  — {}", st, s.name, s.description)).style(style)
            })
            .collect();

        f.render_widget(
            List::new(items)
                .block(Block::default().borders(Borders::ALL))
                .highlight_style(Style::default().add_modifier(Modifier::BOLD)),
            chunks[1],
        );

        let status = if !self.status.is_empty() {
            format!("  |  {}", self.status)
        } else {
            String::new()
        };
        f.render_widget(
            Paragraph::new(format!(
                "↑↓ Navigate  Enter View  r Refresh Registry  Esc Back{}",
                status
            ))
            .block(Block::default().borders(Borders::ALL)),
            chunks[2],
        );
    }

    fn render_skill_detail(&self, f: &mut Frame, area: Rect, idx: usize) {
        if let Some(skill) = self.skills_list.get(idx) {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(1), Constraint::Length(3)])
                .split(area);

            let content = std::fs::read_to_string(skill.path.join("SKILL.md")).unwrap_or_default();
            let lines: Vec<Line> = content
                .lines()
                .map(|l| {
                    if l.starts_with("---") {
                        Line::from(Span::styled(l, Style::default().fg(Color::DarkGray)))
                    } else if l.starts_with("#") {
                        Line::from(Span::styled(
                            l,
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        ))
                    } else {
                        Line::from(l)
                    }
                })
                .collect();
            f.render_widget(
                Paragraph::new(lines).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(skill.name.clone()),
                ),
                chunks[0],
            );
            f.render_widget(
                Paragraph::new("Esc: Back").block(Block::default().borders(Borders::ALL)),
                chunks[1],
            );
        }
    }

    fn render_memory(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(3)])
            .split(area);

        f.render_widget(
            Paragraph::new("🧠 mneme Memory Browser")
                .style(Style::default().add_modifier(Modifier::BOLD))
                .block(Block::default().borders(Borders::ALL)),
            chunks[0],
        );

        let has_mneme = mneme::find_mneme().is_some();
        let body = if has_mneme {
            vec![
                Line::from("Browse mneme memories from the terminal:"),
                Line::from(""),
                Line::from(Span::styled(
                    "  mneme list --project <name>",
                    Style::default().fg(Color::Cyan),
                )),
                Line::from(Span::styled(
                    "  mneme search <query> --project <name>",
                    Style::default().fg(Color::Cyan),
                )),
                Line::from(Span::styled(
                    "  mneme stats --project <name>",
                    Style::default().fg(Color::Cyan),
                )),
                Line::from(Span::styled(
                    "  mneme context --project <name>",
                    Style::default().fg(Color::Cyan),
                )),
                Line::from(Span::styled(
                    "  mneme tui",
                    Style::default().fg(Color::Cyan),
                )),
                Line::from(""),
                Line::from("Full TUI memory browser coming in v0.6.0"),
            ]
        } else {
            vec![
                Line::from(Span::styled(
                    "mneme-brain not found on PATH",
                    Style::default().fg(Color::Red),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "Install: cargo install mneme-brain",
                    Style::default().fg(Color::Cyan),
                )),
            ]
        };
        f.render_widget(
            Paragraph::new(body).block(Block::default().borders(Borders::ALL)),
            chunks[1],
        );
    }

    fn render_backups(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(3)])
            .split(area);

        f.render_widget(
            Paragraph::new("💾 Backup Management")
                .style(Style::default().add_modifier(Modifier::BOLD))
                .block(Block::default().borders(Borders::ALL)),
            chunks[0],
        );

        let backup_dir = crate::config_dir().join("backups");
        let mut entries: Vec<String> = Vec::new();
        if backup_dir.exists() {
            if let Ok(dir) = std::fs::read_dir(&backup_dir) {
                for entry in dir.flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                    entries.push(format!("  {}  ({} bytes)", name, size));
                }
            }
        }
        if entries.is_empty() {
            entries.push("  No backups found.".to_string());
        }

        let mut body = vec![Line::from("Backups:")];
        for e in entries {
            body.push(Line::from(e));
        }
        body.push(Line::from(""));
        body.push(Line::from(Span::styled(
            "  CLI: mneme-ai backup create  |  mneme-ai backup list",
            Style::default().fg(Color::Cyan),
        )));

        f.render_widget(
            Paragraph::new(body).block(Block::default().borders(Borders::ALL)),
            chunks[1],
        );
    }

    fn render_help(&self, f: &mut Frame, area: Rect) {
        let help_text = vec![
            Line::from(Span::styled(
                "mneme-ai v0.5.0 — Ecosystem Configurator",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "TUI Navigation",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from("  1-6: Switch tabs    ↑↓/jk: Navigate    Enter: Select"),
            Line::from("  n: Create profile   d: Delete    r: Refresh    Esc: Back    q: Quit"),
            Line::from(""),
            Line::from(Span::styled(
                "Integration with mneme-brain",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from("  mneme-ai install opencode — Configure agent with mneme MCP"),
            Line::from("  mneme-ai install opencode --with-agents — Full setup + orchestrator"),
            Line::from("  mneme search <query>   mneme save --project <p> <title> <content>"),
            Line::from(""),
            Line::from(Span::styled(
                "Integration with mneme-guardian",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from("  mneme-g init      — Create config"),
            Line::from("  mneme-g install   — Pre-commit hook"),
            Line::from("  mneme-g run       — Review staged files"),
            Line::from("  Results auto-save to mneme brain"),
            Line::from(""),
            Line::from(Span::styled(
                "SDD Profiles",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from("  profile create --name cheap --model opencode/default"),
            Line::from("  sync --opencode — Write agents to OpenCode"),
            Line::from(""),
            Line::from(Span::styled(
                "Install entire ecosystem",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from("  cargo install mneme-brain mneme-ai mneme-guardian"),
            Line::from("  mneme-ai tui → Press 5: Full Setup"),
        ];
        f.render_widget(
            Paragraph::new(help_text)
                .block(Block::default().borders(Borders::ALL).title("❓ Help")),
            area,
        );
    }
}

fn which(cmd: &str) -> bool {
    std::process::Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
