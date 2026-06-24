use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Gauge, List, ListItem, Paragraph, Row, Table, Tabs},
    Frame, Terminal,
};
use std::io;

use crate::profile::{ModelAssignment, ProfileStore, SddProfile, SDD_PHASES};

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

    let mut app = App::new(store);
    let res = app.run(&mut terminal);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    res
}

enum Screen {
    Dashboard,
    Profiles,
    ProfileDetail(usize),
    CreateProfile,
    Memory,
    Help,
}

struct App {
    store: ProfileStore,
    profiles: Vec<SddProfile>,
    selected: usize,
    screen: Screen,
    tab_index: usize,
    status: String,
    // Create profile form
    form_name: String,
    form_provider: String,
    form_model: String,
    form_step: usize,
}

const TAB_TITLES: &[&str] = &["📊 Dashboard", "⚙ Profiles", "🧠 Memory", "❓ Help"];

impl App {
    fn new(store: ProfileStore) -> Self {
        let profiles = store.list().unwrap_or_default();
        Self {
            store,
            profiles,
            selected: 0,
            screen: Screen::Dashboard,
            tab_index: 0,
            status: String::new(),
            form_name: String::new(),
            form_provider: String::new(),
            form_model: String::new(),
            form_step: 0,
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
            Screen::Dashboard => match key {
                KeyCode::Char('1') => {
                    self.screen = Screen::Profiles;
                    self.tab_index = 1;
                    true
                }
                KeyCode::Char('2') => {
                    self.screen = Screen::Memory;
                    self.tab_index = 2;
                    true
                }
                KeyCode::Char('3') => {
                    self.screen = Screen::Help;
                    self.tab_index = 3;
                    true
                }
                KeyCode::Char('q') | KeyCode::Esc => false,
                _ => true,
            },
            Screen::Profiles => self.handle_profile_list(key),
            Screen::ProfileDetail(_) => match key {
                KeyCode::Esc | KeyCode::Backspace => {
                    self.screen = Screen::Profiles;
                    true
                }
                KeyCode::Char('q') => false,
                _ => true,
            },
            Screen::CreateProfile => self.handle_create_form(key),
            Screen::Memory => match key {
                KeyCode::Esc | KeyCode::Backspace => {
                    self.screen = Screen::Dashboard;
                    self.tab_index = 0;
                    true
                }
                KeyCode::Char('q') => false,
                _ => true,
            },
            Screen::Help => match key {
                KeyCode::Esc | KeyCode::Backspace => {
                    self.screen = Screen::Dashboard;
                    self.tab_index = 0;
                    true
                }
                KeyCode::Char('q') => false,
                _ => true,
            },
        }
    }

    fn handle_profile_list(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Up | KeyCode::Char('k') => {
                self.selected = self.selected.saturating_sub(1);
                true
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.selected = self
                    .selected
                    .saturating_add(1)
                    .min(self.profiles.len().saturating_sub(1));
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
                let name = self.profiles.get(self.selected).map(|p| p.name.clone());
                if let Some(ref n) = name {
                    if n != "default" {
                        self.store.delete(n).ok();
                        self.profiles = self.store.list().unwrap_or_default();
                        self.status = format!("Deleted: {}", n);
                    }
                }
                true
            }
            KeyCode::Esc | KeyCode::Backspace => {
                self.screen = Screen::Dashboard;
                self.tab_index = 0;
                true
            }
            KeyCode::Char('q') => false,
            _ => true,
        }
    }

    fn handle_create_form(&mut self, key: KeyCode) -> bool {
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
                    let profile = SddProfile {
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
                    };
                    self.store.save(&profile).ok();
                    self.profiles = self.store.list().unwrap_or_default();
                    self.status = format!("Created: {}", profile.name);
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

        // Tabs
        let titles: Vec<&str> = TAB_TITLES.iter().map(|t| *t).collect();
        let tabs = Tabs::new(titles)
            .select(self.tab_index)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("mneme-ai v0.4.0"),
            )
            .style(Style::default().fg(Color::White))
            .highlight_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            );
        f.render_widget(tabs, chunks[0]);

        // Content
        match self.screen {
            Screen::Dashboard => self.render_dashboard(f, chunks[1]),
            Screen::Profiles => self.render_profiles(f, chunks[1]),
            Screen::ProfileDetail(idx) => self.render_profile_detail(f, chunks[1], idx),
            Screen::CreateProfile => self.render_create_form(f, chunks[1]),
            Screen::Memory => self.render_memory(f, chunks[1]),
            Screen::Help => self.render_help(f, chunks[1]),
        }
    }

    fn render_dashboard(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8),
                Constraint::Length(8),
                Constraint::Min(5),
            ])
            .split(area);

        // Status cards
        let cards = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[0]);

        let profile_count = self.profiles.len();
        let default_profile = self.profiles.iter().find(|p| p.name == "default");

        let status_items = vec![
            Line::from(format!("SDD Profiles: {} configured", profile_count)),
            Line::from(format!(
                "Default: {}/{}",
                default_profile
                    .map(|p| p.orchestrator.provider.as_str())
                    .unwrap_or("none"),
                default_profile
                    .map(|p| p.orchestrator.model.as_str())
                    .unwrap_or("none")
            )),
            Line::from(format!("Phases: {} available", SDD_PHASES.len())),
            Line::from(""),
            Line::from(Span::styled(
                "Status: ✅ Operational",
                Style::default().fg(Color::Green),
            )),
        ];
        let status_block = Paragraph::new(status_items).block(
            Block::default()
                .borders(Borders::ALL)
                .title("📊 System Status"),
        );
        f.render_widget(status_block, cards[0]);

        let agent_counts = format!("Supported Agents: 11");
        let quick_actions = vec![
            Line::from(Span::raw(agent_counts)),
            Line::from(""),
            Line::from(Span::styled(
                "Press 1: Profiles",
                Style::default().fg(Color::Cyan),
            )),
            Line::from(Span::styled(
                "Press 2: Memory",
                Style::default().fg(Color::Cyan),
            )),
            Line::from(Span::styled(
                "Press 3: Help",
                Style::default().fg(Color::Cyan),
            )),
            Line::from(Span::styled("q: Quit", Style::default().fg(Color::Red))),
        ];
        let actions_block = Paragraph::new(quick_actions).block(
            Block::default()
                .borders(Borders::ALL)
                .title("⚡ Quick Actions"),
        );
        f.render_widget(actions_block, cards[1]);

        // Profile preview
        if !self.profiles.is_empty() {
            let rows: Vec<Row> = self
                .profiles
                .iter()
                .map(|p| {
                    let phases = if p.phases.is_empty() {
                        "all default".to_string()
                    } else {
                        format!("{} overrides", p.phases.len())
                    };
                    Row::new(vec![
                        Cell::from(p.name.clone()),
                        Cell::from(format!(
                            "{}/{}",
                            p.orchestrator.provider, p.orchestrator.model
                        )),
                        Cell::from(phases),
                    ])
                })
                .collect();

            let widths = [
                Constraint::Length(20),
                Constraint::Length(30),
                Constraint::Length(15),
            ];
            let table = Table::new(rows, widths)
                .header(
                    Row::new(vec!["Name", "Orchestrator", "Phases"])
                        .style(Style::default().add_modifier(Modifier::BOLD)),
                )
                .block(Block::default().borders(Borders::ALL).title("Profiles"));
            f.render_widget(table, chunks[1]);
        } else {
            let no_profiles = Paragraph::new("No profiles yet. Create one:\n  mneme-ai profile create --name cheap --model opencode/default\n\nOr press 1 to manage profiles.")
                .block(Block::default().borders(Borders::ALL).title("Profiles"));
            f.render_widget(no_profiles, chunks[1]);
        }

        // CLI reference
        let help = Paragraph::new(vec![
            Line::from("mneme-ai commands:"),
            Line::from("  init         Initialize config"),
            Line::from("  install      Install agent integration"),
            Line::from("  doctor       Health check"),
            Line::from("  profile      Manage SDD profiles"),
            Line::from("  sync         Sync to OpenCode"),
            Line::from("  backup       Backup/restore configs"),
        ])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("CLI Reference"),
        );
        f.render_widget(help, chunks[2]);
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
                    " {}  [{}/{}]  {}  ",
                    p.name, p.orchestrator.provider, p.orchestrator.model, phases
                ))
                .style(style)
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("⚙ SDD Profiles"),
            )
            .highlight_style(Style::default().add_modifier(Modifier::BOLD));
        f.render_widget(list, chunks[0]);

        let msg = if !self.status.is_empty() {
            format!("  |  {}", self.status)
        } else {
            String::new()
        };
        let help = Paragraph::new(format!(
            "↑↓/jk Navigate  Enter View  n New  d Delete  Esc Back  q Quit{}",
            msg
        ))
        .block(Block::default().borders(Borders::ALL));
        f.render_widget(help, chunks[1]);
    }

    fn render_profile_detail(&self, f: &mut Frame, area: Rect, idx: usize) {
        if let Some(profile) = self.profiles.get(idx) {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(5),
                    Constraint::Length(3),
                ])
                .split(area);

            let header = Paragraph::new(format!(
                "📋 {}  |  {}/{}  |  {} phases",
                profile.name,
                profile.orchestrator.provider,
                profile.orchestrator.model,
                if profile.phases.is_empty() {
                    0
                } else {
                    profile.phases.len()
                }
            ))
            .style(Style::default().add_modifier(Modifier::BOLD))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Profile Detail"),
            );
            f.render_widget(header, chunks[0]);

            let mut rows = vec![Row::new(vec!["Phase", "Provider", "Model", "Source"])
                .style(Style::default().add_modifier(Modifier::BOLD))];

            // Orchestrator row
            rows.push(
                Row::new(vec![
                    "🔄 orchestrator",
                    &profile.orchestrator.provider,
                    &profile.orchestrator.model,
                    "profile",
                ])
                .style(Style::default().fg(Color::Cyan)),
            );

            // SDD phases
            for phase in SDD_PHASES {
                let has_override = profile.phases.contains_key(*phase);
                let (provider, model) = if let Some(a) = profile.phases.get(*phase) {
                    (a.provider.as_str(), a.model.as_str())
                } else {
                    (
                        profile.orchestrator.provider.as_str(),
                        profile.orchestrator.model.as_str(),
                    )
                };
                let source = if has_override {
                    "override"
                } else {
                    "inherited"
                };
                rows.push(
                    Row::new(vec![*phase, provider, model, source]).style(if has_override {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    }),
                );
            }

            let widths = [
                Constraint::Length(20),
                Constraint::Length(15),
                Constraint::Length(20),
                Constraint::Length(12),
            ];
            let table = Table::new(rows, widths).block(Block::default().borders(Borders::ALL));
            f.render_widget(table, chunks[1]);

            let help = Paragraph::new("Esc: Back").block(Block::default().borders(Borders::ALL));
            f.render_widget(help, chunks[2]);
        }
    }

    fn render_create_form(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(5),
                Constraint::Length(3),
            ])
            .split(area);

        let header = Paragraph::new("Create New SDD Profile")
            .style(Style::default().add_modifier(Modifier::BOLD))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(header, chunks[0]);

        let indicator = |s: usize| if self.form_step == s { "◄" } else { "✓" };
        let fields = vec![
            Line::from(format!("Profile name [a-z, hyphens]: {} {}", self.form_name, indicator(0))),
            Line::from(format!("Provider [default: opencode]: {} {}", self.form_provider, indicator(1))),
            Line::from(format!("Model [default: default]: {} {}", self.form_model, indicator(2))),
            Line::from(""),
            Line::from(Span::styled("Tab/Enter: Next  |  Esc: Cancel", Style::default().fg(Color::Cyan))),
            Line::from(Span::styled("Create profiles via CLI: mneme-ai profile create --name <name> --model <provider/model>", Style::default().fg(Color::DarkGray))),
        ];
        let text = Paragraph::new(fields).block(Block::default().borders(Borders::ALL));
        f.render_widget(text, chunks[1]);
    }

    fn render_memory(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(5)])
            .split(area);

        let header = Paragraph::new("🧠 mneme Memory Browser")
            .style(Style::default().add_modifier(Modifier::BOLD))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(header, chunks[0]);

        let body = Paragraph::new(vec![
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
            Line::from(""),
            Line::from("Memory browser TUI coming in v0.5.0"),
        ])
        .block(Block::default().borders(Borders::ALL));
        f.render_widget(body, chunks[1]);
    }

    fn render_help(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(5)])
            .split(area);

        let header = Paragraph::new("❓ Help — mneme-ai v0.4.0")
            .style(Style::default().add_modifier(Modifier::BOLD))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(header, chunks[0]);

        let help = Paragraph::new(vec![
            Line::from(Span::styled(
                "TUI Navigation",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from("  1-3: Switch tabs    ↑↓/jk: Navigate    Enter: Select"),
            Line::from("  n: Create profile   d: Delete profile    Esc: Back    q: Quit"),
            Line::from(""),
            Line::from(Span::styled(
                "CLI Commands",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from("  mneme-ai init        — Initialize config"),
            Line::from("  mneme-ai install <a> — Install agent (opencode, claude-code, etc.)"),
            Line::from("  mneme-ai doctor      — Health check"),
            Line::from("  mneme-ai tui         — Launch this TUI"),
            Line::from(""),
            Line::from(Span::styled(
                "Profile Management",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from("  profile create --name cheap --model opencode/default"),
            Line::from("  profile list        profile show <name>"),
            Line::from("  profile edit --name <n> --model <p/m>"),
            Line::from("  profile clone <src> <dst>"),
            Line::from("  profile delete <name>"),
            Line::from(""),
            Line::from(Span::styled(
                "Integration",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from("  sync --opencode     — Write agents to opencode.json"),
            Line::from("  backup create       — Backup configurations"),
            Line::from("  install opencode --with-agents — Full setup"),
        ])
        .block(Block::default().borders(Borders::ALL));
        f.render_widget(help, chunks[1]);
    }
}
