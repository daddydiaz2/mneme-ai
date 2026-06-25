//! Complete wizard TUI for mneme-ai — gentle-ai style but better.
use crate::mneme;
use crate::profile::{ModelAssignment, ProfileStore, SddProfile, SDD_PHASES};
use crate::skills;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, List, ListItem, Paragraph, Row, Table, Tabs},
    Frame, Terminal,
};
use std::io;

type Term = Terminal<ratatui::backend::CrosstermBackend<io::Stdout>>;

const PROVIDERS: &[&str] = &[
    "opencode",
    "anthropic",
    "openai",
    "google",
    "ollama",
    "github",
];
const MODELS: &[(&str, &[&str])] = &[
    ("opencode", &["deepseek-v4-flash", "deepseek-v4-pro"]),
    (
        "anthropic",
        &[
            "claude-sonnet-4-20250514",
            "claude-haiku-3.5-20241022",
            "claude-opus-4-20250514",
        ],
    ),
    ("openai", &["gpt-5-mini", "gpt-5", "o4-mini", "o4"]),
    ("google", &["gemini-2.5-pro", "gemini-2.5-flash"]),
    (
        "ollama",
        &[
            "qwen2.5-coder:7b",
            "qwen2.5-coder:32b",
            "deepseek-coder:6.7b",
        ],
    ),
    ("github", &["gpt-5-mini", "claude-sonnet-4"]),
];
const REASONING_EFFORTS: &[&str] = &["default", "low", "medium", "high"];

pub fn run_tui() -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut s = io::stdout();
    execute!(s, EnterAlternateScreen)?;
    let mut t = Terminal::new(ratatui::backend::CrosstermBackend::new(s))?;
    t.clear()?;
    let store = ProfileStore::new();
    let _ = store.init();
    let _ = skills::install_mneme_skills();
    let mut a = Wizard::new(store);
    let r = a.run(&mut t);
    disable_raw_mode()?;
    execute!(t.backend_mut(), LeaveAlternateScreen)?;
    t.show_cursor()?;
    r
}

struct Profile {
    project: String,
    provider: String,
    model: String,
    effort: usize,
    phases: Vec<(String, String, String, usize)>, // name, provider, model, effort
}

struct Wizard {
    store: ProfileStore,
    step: usize,
    profiles: Vec<Profile>,
    skills: Vec<skills::Skill>,
    logs: Vec<String>,
    sel_agent: usize,
    agents: Vec<(&'static str, bool)>,
    sel_provider: usize,
    sel_model: usize,
    sel_effort: usize,
    create_profile: bool,
    profile_name: String,
    phase_sel: usize,
    customizing_phase: bool,
}

impl Wizard {
    fn new(store: ProfileStore) -> Self {
        let agents: Vec<_> = crate::agents::SUPPORTED_AGENTS
            .iter()
            .filter(|a| a.supported)
            .map(|a| (a.name, false))
            .collect();
        Self {
            store,
            step: 0,
            profiles: vec![Profile {
                project: "default".into(),
                provider: "opencode".into(),
                model: "deepseek-v4-flash".into(),
                effort: 0,
                phases: vec![],
            }],
            skills: skills::scan_skills(),
            logs: vec![],
            sel_agent: 0,
            agents,
            sel_provider: 0,
            sel_model: 0,
            sel_effort: 0,
            create_profile: false,
            profile_name: "default".into(),
            phase_sel: 0,
            customizing_phase: false,
        }
    }
    fn run(&mut self, t: &mut Term) -> anyhow::Result<()> {
        loop {
            t.draw(|f| self.draw(f))?;
            if let Event::Key(k) = event::read()? {
                if !self.key(k.code) {
                    break;
                }
            }
        }
        Ok(())
    }
    fn key(&mut self, k: KeyCode) -> bool {
        let step = self.step;
        match step {
            0 => match k {
                KeyCode::Char('1') => self.step = 1,
                KeyCode::Char('2') => {
                    self.step = 5;
                    self.run_all();
                }
                KeyCode::Char('q') | KeyCode::Esc => return false,
                _ => {}
            },
            1 => match k {
                KeyCode::Up | KeyCode::Char('k') => {
                    self.sel_agent = self.sel_agent.saturating_sub(1)
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.sel_agent = self.sel_agent.saturating_add(1).min(self.agents.len() - 1)
                }
                KeyCode::Char(' ') => {
                    if let Some(a) = self.agents.get_mut(self.sel_agent) {
                        a.1 = !a.1;
                    }
                }
                KeyCode::Char('a') => {
                    let all = self.agents.iter().any(|a| !a.1);
                    for a in &mut self.agents {
                        a.1 = all;
                    }
                }
                KeyCode::Tab | KeyCode::Enter => {
                    if !self.agents.iter().any(|a| a.1) {
                        self.agents[0].1 = true;
                    }
                    self.step = 2;
                }
                KeyCode::Esc => self.step = 0,
                KeyCode::Char('q') => return false,
                _ => {}
            },
            2 => {
                if self.sel_effort == 0 {
                    // Provider selection
                    match k {
                        KeyCode::Up | KeyCode::Char('k') => {
                            self.sel_provider = self.sel_provider.saturating_sub(1)
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            self.sel_provider =
                                self.sel_provider.saturating_add(1).min(PROVIDERS.len() - 1)
                        }
                        KeyCode::Enter => {
                            self.sel_effort = 1;
                            self.sel_model = 0;
                        }
                        KeyCode::Esc => self.step = 1,
                        KeyCode::Char('q') => return false,
                        _ => {}
                    }
                } else if self.sel_effort == 1 {
                    // Model selection
                    let provider = PROVIDERS[self.sel_provider];
                    let models_opt = MODELS.iter().find(|(n, _)| *n == provider);
                    let mcount = models_opt.map(|(_, m)| m.len()).unwrap_or(0);
                    match k {
                        KeyCode::Up | KeyCode::Char('k') => {
                            self.sel_model = self.sel_model.saturating_sub(1)
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            self.sel_model = self
                                .sel_model
                                .saturating_add(1)
                                .min(mcount.saturating_sub(1))
                        }
                        KeyCode::Enter => {
                            if let Some(models) = models_opt {
                                if !models.1.is_empty() && self.sel_model < models.1.len() {
                                    self.profiles[0].provider = PROVIDERS[self.sel_provider].into();
                                    self.profiles[0].model = models.1[self.sel_model].into();
                                }
                            }
                            self.sel_effort = 2;
                        }
                        KeyCode::Esc => self.sel_effort = 0,
                        KeyCode::Char('q') => return false,
                        _ => {}
                    }
                } else {
                    // Reasoning effort
                    match k {
                        KeyCode::Char('1') => {
                            self.profiles[0].effort = 1;
                            self.step = if self.create_profile { 3 } else { 4 };
                        }
                        KeyCode::Char('2') => {
                            self.profiles[0].effort = 2;
                            self.step = if self.create_profile { 3 } else { 4 };
                        }
                        KeyCode::Char('3') => {
                            self.profiles[0].effort = 3;
                            self.step = if self.create_profile { 3 } else { 4 };
                        }
                        KeyCode::Enter => {
                            self.step = if self.create_profile { 3 } else { 4 };
                        }
                        KeyCode::Esc => self.sel_effort = 1,
                        KeyCode::Char('q') => return false,
                        _ => {}
                    }
                }
            }
            3 => match k {
                KeyCode::Up | KeyCode::Char('k') => {
                    self.phase_sel = self.phase_sel.saturating_sub(1)
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.phase_sel = self.phase_sel.saturating_add(1).min(SDD_PHASES.len())
                }
                KeyCode::Tab | KeyCode::Enter => {
                    self.step = 4;
                }
                KeyCode::Esc => self.step = 2,
                KeyCode::Char('q') => return false,
                _ => {}
            },
            4 => match k {
                KeyCode::Tab | KeyCode::Enter => self.run_all(),
                KeyCode::Esc => self.step = if self.create_profile { 3 } else { 2 },
                KeyCode::Char('q') => return false,
                _ => {}
            },
            5 => {
                if k == KeyCode::Esc || k == KeyCode::Char('q') {
                    return false;
                }
            }
            _ => {}
        };
        true
    }
    fn run_all(&mut self) {
        self.logs.clear();
        self.logs.push("🚀 Running setup...".into());
        for (name, sel) in &self.agents {
            if *sel {
                self.logs.push(format!("  Installing {}...", name));
                let _ = crate::install::install_agent(name);
                self.logs.push(format!("✓ {} configured", name));
            }
        }
        self.logs.push("  Creating orchestrator agents...".into());
        let _ = crate::install::install_opencode_agents();
        self.logs.push("  Installing skills & branding...".into());
        let _ = skills::install_mneme_skills();
        let _ = crate::opencode::customize_opencode();
        self.skills = skills::scan_skills();
        if self.create_profile {
            let p = &self.profiles[0];
            let mut phases = std::collections::HashMap::new();
            for (n, prov, mdl, _) in &p.phases {
                phases.insert(
                    n.clone(),
                    ModelAssignment {
                        provider: prov.clone(),
                        model: mdl.clone(),
                        reasoning_effort: Some(REASONING_EFFORTS[p.effort].into()),
                    },
                );
            }
            let _ = self.store.save(&SddProfile {
                name: self.profile_name.clone(),
                orchestrator: ModelAssignment {
                    provider: p.provider.clone(),
                    model: p.model.clone(),
                    reasoning_effort: Some(REASONING_EFFORTS[p.effort].into()),
                },
                phases,
            });
            self.logs
                .push(format!("✓ Profile '{}' created", self.profile_name));
        }
        self.logs.push("✅ Setup complete!".into());
        self.step = 5;
    }
    fn draw(&self, f: &mut Frame) {
        let area = f.size();
        let h = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(1)])
            .split(area);
        let tabs = [
            "🏠 Welcome",
            "💻 Agents",
            "⚙ Model",
            "📋 Phases",
            "▶ Execute",
            "✅ Done",
        ];
        let t = Tabs::new(tabs.to_vec())
            .select(self.step)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("mneme-ai Wizard"),
            )
            .style(Style::default().fg(Color::White))
            .highlight_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            );
        f.render_widget(t, h[0]);
        match self.step {
            0 => self.welcome(f, h[1]),
            1 => self.pick_agents(f, h[1]),
            2 => self.pick_model(f, h[1]),
            3 => self.pick_phases(f, h[1]),
            4 => self.review(f, h[1]),
            5 => self.done(f, h[1]),
            _ => {}
        }
    }
    fn welcome(&self, f: &mut Frame, a: Rect) {
        let c = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(a);
        let brain = mneme::find_mneme().is_some();
        f.render_widget(
            Paragraph::new(vec![
                Line::from(Span::styled(
                    if brain {
                        "🧠 mneme-brain ✅"
                    } else {
                        "🧠 mneme-brain ❌"
                    },
                    Style::default().fg(if brain { Color::Green } else { Color::Red }),
                )),
                Line::from(format!(
                    "SDD Profiles: {}",
                    self.store.list().unwrap_or_default().len()
                )),
                Line::from(format!("Skills: {}", self.skills.len())),
                Line::from(format!("Agents: {}", self.agents.len())),
            ])
            .block(Block::default().borders(Borders::ALL).title("📊 Status")),
            c[0],
        );
        f.render_widget(
            Paragraph::new(vec![
                Line::from(Span::styled(
                    "1: Start Setup Wizard",
                    Style::default().fg(Color::Cyan),
                )),
                Line::from(Span::styled(
                    "2: Quick Setup (all default)",
                    Style::default().fg(Color::Cyan),
                )),
                Line::from(""),
                Line::from(Span::styled("q: Quit", Style::default().fg(Color::Red))),
            ])
            .block(Block::default().borders(Borders::ALL).title("⚡ Actions")),
            c[1],
        );
    }
    fn pick_agents(&self, f: &mut Frame, a: Rect) {
        let c = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(3)])
            .split(a);
        let items: Vec<ListItem> = self
            .agents
            .iter()
            .enumerate()
            .map(|(i, (n, s))| {
                let st = if i == self.sel_agent {
                    Style::default().bg(Color::Blue).fg(Color::White)
                } else {
                    Style::default()
                };
                ListItem::new(format!(" [{}] {}", if *s { "✓" } else { " " }, n)).style(st)
            })
            .collect();
        f.render_widget(
            List::new(items).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("💻 Select Agents (Space: toggle, a: all, Tab: next)"),
            ),
            c[0],
        );
        f.render_widget(
            Paragraph::new(Span::styled(
                "↑↓: Navigate  Space: Select  a: All/None  Tab: Continue  Esc: Back",
                Style::default().fg(Color::DarkGray),
            ))
            .block(Block::default().borders(Borders::ALL)),
            c[1],
        );
    }
    fn pick_model(&self, f: &mut Frame, a: Rect) {
        let c = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(3)])
            .split(a);
        let title = match self.sel_effort {
            0 => "⚙ Select Provider",
            1 => "🔧 Select Model",
            _ => "🎯 Reasoning Effort (1: low 2: medium 3: high, Enter: default)",
        };
        if self.sel_effort == 0 {
            let items: Vec<ListItem> = PROVIDERS
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    let st = if i == self.sel_provider {
                        Style::default().bg(Color::Blue).fg(Color::White)
                    } else {
                        Style::default()
                    };
                    let models = MODELS
                        .iter()
                        .find(|(n, _)| n == p)
                        .map(|(_, m)| m.join(", "))
                        .unwrap_or_default();
                    ListItem::new(format!(" {}  — models: {}", p, models)).style(st)
                })
                .collect();
            f.render_widget(
                List::new(items).block(Block::default().borders(Borders::ALL).title(title)),
                c[0],
            );
            f.render_widget(
                Paragraph::new(Span::styled(
                    "↑↓: Choose provider  Enter: Next  Esc: Back",
                    Style::default().fg(Color::DarkGray),
                ))
                .block(Block::default().borders(Borders::ALL)),
                c[1],
            );
        } else if self.sel_effort == 1 {
            let provider = PROVIDERS[self.sel_provider];
            let model_list: Vec<&str> = MODELS
                .iter()
                .find(|(n, _)| *n == provider)
                .map(|(_, m)| m.to_vec())
                .unwrap_or_default();
            let models = model_list;
            let items: Vec<ListItem> = models
                .iter()
                .enumerate()
                .map(|(i, m)| {
                    let st = if i == self.sel_model {
                        Style::default().bg(Color::Blue).fg(Color::White)
                    } else {
                        Style::default()
                    };
                    ListItem::new(format!(" {}", m)).style(st)
                })
                .collect();
            f.render_widget(
                List::new(items).block(Block::default().borders(Borders::ALL).title(title)),
                c[0],
            );
            f.render_widget(
                Paragraph::new(Span::styled(
                    "↑↓: Choose model  Enter: Confirm  Esc: Back",
                    Style::default().fg(Color::DarkGray),
                ))
                .block(Block::default().borders(Borders::ALL)),
                c[1],
            );
        } else {
            let effort_items = vec![
                ListItem::new(" 1: Low (fast, cheaper)"),
                ListItem::new(" 2: Medium (balanced)"),
                ListItem::new(" 3: High (best quality)"),
                ListItem::new(" Enter: Default (provider default)"),
            ];
            f.render_widget(
                List::new(effort_items).block(Block::default().borders(Borders::ALL).title(title)),
                c[0],
            );
            f.render_widget(
                Paragraph::new(Span::styled(
                    "1-3: Select effort  Enter: Default  Esc: Back",
                    Style::default().fg(Color::DarkGray),
                ))
                .block(Block::default().borders(Borders::ALL)),
                c[1],
            );
        }
    }
    fn pick_phases(&self, f: &mut Frame, a: Rect) {
        let c = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(3)])
            .split(a);
        let rows: Vec<Row> = SDD_PHASES
            .iter()
            .enumerate()
            .map(|(i, ph)| {
                let prov = &self.profiles[0].provider;
                let mdl = &self.profiles[0].model;
                Row::new(vec![
                    if self.phase_sel == i { "→" } else { " " },
                    ph,
                    prov,
                    mdl,
                ])
                .style(if i == self.phase_sel {
                    Style::default().bg(Color::Blue).fg(Color::White)
                } else {
                    Style::default()
                })
            })
            .collect();
        f.render_widget(
            Table::new(
                rows,
                [
                    Constraint::Length(3),
                    Constraint::Length(20),
                    Constraint::Length(15),
                    Constraint::Length(25),
                ],
            )
            .header(
                Row::new(vec!["", "Phase", "Provider", "Model"])
                    .style(Style::default().add_modifier(Modifier::BOLD)),
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("📋 SDD Phase Models (Tab: continue)"),
            ),
            c[0],
        );
        f.render_widget(
            Paragraph::new(Span::styled(
                "↑↓: Navigate  Tab: Continue to Execute  Esc: Back",
                Style::default().fg(Color::DarkGray),
            ))
            .block(Block::default().borders(Borders::ALL)),
            c[1],
        );
    }
    fn review(&self, f: &mut Frame, a: Rect) {
        let c = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(3)])
            .split(a);
        let mut lines = vec![
            Line::from(Span::styled(
                "Review your configuration:",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(format!(
                "  Agents ({}): {}",
                self.agents.iter().filter(|a| a.1).count(),
                self.agents
                    .iter()
                    .filter(|a| a.1)
                    .map(|a| a.0)
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
            Line::from(format!(
                "  Provider: {}  |  Model: {}",
                self.profiles[0].provider, self.profiles[0].model
            )),
            if self.create_profile {
                Line::from(format!(
                    "  Profile: {}  |  Phases: {}",
                    self.profile_name,
                    SDD_PHASES.len()
                ))
            } else {
                Line::from("  Profile: none (will use defaults)")
            },
            Line::from(""),
            Line::from(Span::styled(
                "  Tab/Enter: Execute setup  |  Esc: Back  |  q: Quit",
                Style::default().fg(Color::Cyan),
            )),
        ];
        if !self.logs.is_empty() {
            lines.push(Line::from(""));
            for l in &self.logs {
                lines.push(Line::from(Span::styled(
                    l.clone(),
                    Style::default().fg(Color::Green),
                )));
            }
        }
        f.render_widget(
            Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title("▶ Execute")),
            c[0],
        );
        f.render_widget(
            Paragraph::new("Tab: Run  Esc: Back  q: Quit")
                .block(Block::default().borders(Borders::ALL)),
            c[1],
        );
    }
    fn done(&self, f: &mut Frame, a: Rect) {
        let lines: Vec<Line> = self
            .logs
            .iter()
            .enumerate()
            .map(|(_, l)| {
                let style = if l.starts_with("✅") {
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD)
                } else if l.starts_with("✓") {
                    Style::default().fg(Color::Green)
                } else if l.starts_with("🚀") {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                Line::from(Span::styled(l.clone(), style))
            })
            .collect();
        f.render_widget(
            Paragraph::new(lines).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("✅ Setup Complete"),
            ),
            a,
        );
    }
}
