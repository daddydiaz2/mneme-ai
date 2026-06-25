//! Complete wizard TUI for mneme-ai — gentle-ai style with per-phase config.
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
    widgets::{Block, Borders, List, ListItem, Paragraph, Row, Table},
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
const EFFORTS: &[&str] = &["default", "low", "medium", "high", "xhigh"];

fn models_for_provider(provider: &str) -> Vec<&'static str> {
    for (p, models) in MODELS {
        if *p == provider {
            return models.to_vec();
        }
    }
    vec![]
}

pub fn run_tui() -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut s = io::stdout();
    execute!(s, EnterAlternateScreen)?;
    let mut t = Terminal::new(ratatui::backend::CrosstermBackend::new(s))?;
    t.clear()?;
    let store = ProfileStore::new();
    let _ = store.init();
    let _ = skills::install_mneme_skills();
    let mut w = Wizard::new(store);
    let r = w.run(&mut t);
    disable_raw_mode()?;
    execute!(t.backend_mut(), LeaveAlternateScreen)?;
    t.show_cursor()?;
    r
}

struct Wizard {
    store: ProfileStore,
    step: usize,
    logs: Vec<String>,
    agents: Vec<(&'static str, bool)>,
    sel_agent: usize,
    provider: String,
    model: String,
    effort: String,
    sel_provider: usize,
    sel_model: usize,
    sel_effort: usize,
    phase_models: Vec<(String, String, String)>, // phase, provider, model
    phase_sel: usize,
    edit_phase: bool,
    profiles: Vec<skills::Skill>,
    create_profile: bool,
}

impl Wizard {
    fn new(store: ProfileStore) -> Self {
        let agents: Vec<_> = crate::agents::SUPPORTED_AGENTS
            .iter()
            .filter(|a| a.supported)
            .map(|a| (a.name, false))
            .collect();
        let phase_models: Vec<_> = SDD_PHASES
            .iter()
            .map(|p| (p.to_string(), "opencode".to_string(), "default".to_string()))
            .collect();
        Self {
            store,
            step: 0,
            logs: vec![],
            agents,
            sel_agent: 0,
            provider: "opencode".into(),
            model: "deepseek-v4-flash".into(),
            effort: "default".into(),
            sel_provider: 0,
            sel_model: 0,
            sel_effort: 0,
            phase_models,
            phase_sel: 0,
            edit_phase: false,
            profiles: skills::scan_skills(),
            create_profile: true,
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
        match self.step {
            0 => match k {
                KeyCode::Char('1') => self.step = 1,
                KeyCode::Char('2') => {
                    self.step = 5;
                    self.execute();
                }
                KeyCode::Char('q') | KeyCode::Esc => return false,
                _ => {}
            },
            1 => self.key_agents(k),
            2 => self.key_model(k),
            3 => self.key_phases(k),
            4 => {
                if k == KeyCode::Tab || k == KeyCode::Enter {
                    self.execute();
                } else if k == KeyCode::Esc {
                    self.step = 3;
                } else if k == KeyCode::Char('q') {
                    return false;
                }
            }
            5 => {
                if k == KeyCode::Esc || k == KeyCode::Char('q') {
                    return false;
                }
            }
            _ => {}
        }
        true
    }
    fn key_agents(&mut self, k: KeyCode) {
        match k {
            KeyCode::Up | KeyCode::Char('k') => self.sel_agent = self.sel_agent.saturating_sub(1),
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
            _ => {}
        }
    }
    fn key_model(&mut self, k: KeyCode) {
        match self.sel_effort {
            0 => match k {
                // Provider selection
                KeyCode::Up | KeyCode::Char('k') => {
                    self.sel_provider = self.sel_provider.saturating_sub(1)
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.sel_provider = self.sel_provider.saturating_add(1).min(PROVIDERS.len() - 1)
                }
                KeyCode::Enter => {
                    self.sel_effort = 1;
                    self.sel_model = 0;
                }
                KeyCode::Esc => self.step = 1,
                _ => {}
            },
            1 => match k {
                // Model selection
                KeyCode::Up | KeyCode::Char('k') => {
                    self.sel_model = self.sel_model.saturating_sub(1)
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.sel_model = self.sel_model.saturating_add(1)
                }
                KeyCode::Enter => {
                    self.provider = PROVIDERS[self.sel_provider].into();
                    let models = models_for_provider(&self.provider);
                    if self.sel_model < models.len() {
                        self.model = models[self.sel_model].to_string();
                    }
                    self.sel_effort = 2;
                    self.sel_model = 0;
                }
                KeyCode::Esc => self.sel_effort = 0,
                _ => {}
            },
            _ => match k {
                // Effort selection
                KeyCode::Up | KeyCode::Char('k') => {
                    self.sel_model = self.sel_model.saturating_sub(1)
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.sel_model = self.sel_model.saturating_add(1).min(EFFORTS.len() - 1)
                }
                KeyCode::Enter => {
                    self.effort = EFFORTS[self.sel_model].into();
                    self.sel_model = 0;
                    self.sel_effort = 0;
                    // Apply to all phases
                    for p in &mut self.phase_models {
                        p.1 = self.provider.clone();
                        p.2 = self.model.clone();
                    }
                    self.step = 3;
                }
                KeyCode::Esc => self.sel_effort = 1,
                _ => {}
            },
        }
    }
    fn key_phases(&mut self, k: KeyCode) {
        if self.edit_phase {
            // Editing a specific phase's model
            match k {
                KeyCode::Up | KeyCode::Char('k') => {
                    self.sel_provider = self.sel_provider.saturating_sub(1)
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.sel_provider = self.sel_provider.saturating_add(1).min(PROVIDERS.len() - 1)
                }
                KeyCode::Enter => {
                    let phase = &mut self.phase_models[self.phase_sel];
                    phase.1 = PROVIDERS[self.sel_provider].into();
                    let models = models_for_provider(&phase.1);
                    if self.sel_model < models.len() {
                        phase.2 = models[self.sel_model].to_string();
                    }
                    self.edit_phase = false;
                    self.sel_provider = 0;
                    self.sel_model = 0;
                }
                KeyCode::Esc => self.edit_phase = false,
                _ => {}
            }
        } else {
            match k {
                KeyCode::Up | KeyCode::Char('k') => {
                    self.phase_sel = self.phase_sel.saturating_sub(1)
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.phase_sel = self.phase_sel.saturating_add(1).min(SDD_PHASES.len() - 1)
                }
                KeyCode::Enter => {
                    // Enter edit mode for this phase
                    self.edit_phase = true;
                    self.sel_provider = 0;
                    self.sel_model = 0;
                    let p = &self.phase_models[self.phase_sel];
                    // Try to find provider index
                    for (i, prov) in PROVIDERS.iter().enumerate() {
                        if prov == &p.1 {
                            self.sel_provider = i;
                            break;
                        }
                    }
                }
                KeyCode::Tab | KeyCode::Enter if self.phase_sel == SDD_PHASES.len() - 1 => {
                    self.step = 4
                }
                KeyCode::Esc => self.step = 2,
                _ => {}
            }
        }
    }
    fn execute(&mut self) {
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
        let _ = skills::install_mneme_skills();
        let _ = crate::opencode::customize_opencode();
        self.profiles = skills::scan_skills();
        // Save profile
        let mut phases = std::collections::HashMap::new();
        for (n, prov, mdl) in &self.phase_models {
            phases.insert(
                n.clone(),
                ModelAssignment {
                    provider: prov.clone(),
                    model: mdl.clone(),
                    reasoning_effort: Some(self.effort.clone()),
                },
            );
        }
        let _ = self.store.save(&SddProfile {
            name: "default".into(),
            orchestrator: ModelAssignment {
                provider: self.provider.clone(),
                model: self.model.clone(),
                reasoning_effort: Some(self.effort.clone()),
            },
            phases,
        });
        self.logs.push("✓ Profile saved".into());
        self.logs.push("✅ Setup complete!".into());
        self.step = 5;
    }
    fn draw(&self, f: &mut Frame) {
        let area = f.size();
        let h = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(1)])
            .split(area);
        let tab_names = [
            "🏠 Welcome",
            "💻 Agents",
            "⚙ Model",
            "📋 Phases",
            "▶ Execute",
            "✅ Done",
        ];
        let step_display = format!(
            "mneme-ai Wizard  |  [{}] {}",
            self.step + 1,
            tab_names.get(self.step).unwrap_or(&"?")
        );
        f.render_widget(
            Paragraph::new(Span::styled(
                &step_display,
                Style::default().fg(Color::White),
            ))
            .block(Block::default().borders(Borders::ALL).title("🧠 mneme-ai"))
            .style(Style::default().bg(Color::Rgb(22, 27, 34))),
            h[0],
        );
        match self.step {
            0 => self.welcome(f, h[1]),
            1 => self.agents_screen(f, h[1]),
            2 => self.model_screen(f, h[1]),
            3 => self.phases_screen(f, h[1]),
            4 => self.execute_screen(f, h[1]),
            5 => self.done_screen(f, h[1]),
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
                Line::from(format!("Skills: {} installed", self.profiles.len())),
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
                    "2: Quick Setup",
                    Style::default().fg(Color::Cyan),
                )),
                Line::from(Span::styled("q: Quit", Style::default().fg(Color::Red))),
            ])
            .block(Block::default().borders(Borders::ALL).title("⚡ Actions")),
            c[1],
        );
    }
    fn agents_screen(&self, f: &mut Frame, a: Rect) {
        let c = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(3)])
            .split(a);
        let items: Vec<ListItem> = self
            .agents
            .iter()
            .enumerate()
            .map(|(i, (n, s))| {
                ListItem::new(format!(" [{}] {}", if *s { "✓" } else { " " }, n)).style(
                    if i == self.sel_agent {
                        Style::default().bg(Color::Blue).fg(Color::White)
                    } else {
                        Style::default()
                    },
                )
            })
            .collect();
        f.render_widget(
            List::new(items).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("💻 Select Agents (Space: toggle  a: all)"),
            ),
            c[0],
        );
        f.render_widget(
            Paragraph::new("↑↓ Navigate  Space Select  a All/None  Tab Continue")
                .block(Block::default().borders(Borders::ALL)),
            c[1],
        );
    }
    fn model_screen(&self, f: &mut Frame, a: Rect) {
        let c = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(3)])
            .split(a);
        if self.sel_effort == 0 {
            let items: Vec<ListItem> = PROVIDERS
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    ListItem::new(format!(" {}", p)).style(if i == self.sel_provider {
                        Style::default().bg(Color::Blue).fg(Color::White)
                    } else {
                        Style::default()
                    })
                })
                .collect();
            f.render_widget(
                List::new(items).block(Block::default().borders(Borders::ALL).title("⚙ Provider")),
                c[0],
            );
            f.render_widget(
                Paragraph::new("↑↓ Choose  Enter Next")
                    .block(Block::default().borders(Borders::ALL)),
                c[1],
            );
        } else if self.sel_effort == 1 {
            let models = models_for_provider(PROVIDERS[self.sel_provider]);
            let items: Vec<ListItem> = models
                .iter()
                .enumerate()
                .map(|(i, m)| {
                    ListItem::new(format!(" {}", m)).style(if i == self.sel_model {
                        Style::default().bg(Color::Blue).fg(Color::White)
                    } else {
                        Style::default()
                    })
                })
                .collect();
            f.render_widget(
                List::new(items).block(Block::default().borders(Borders::ALL).title("🔧 Model")),
                c[0],
            );
            f.render_widget(
                Paragraph::new("↑↓ Choose  Enter Next")
                    .block(Block::default().borders(Borders::ALL)),
                c[1],
            );
        } else {
            let items: Vec<ListItem> = EFFORTS
                .iter()
                .enumerate()
                .map(|(i, e)| {
                    let note = match *e {
                        "low" => " (fast, cheap)",
                        "medium" => " (balanced)",
                        "high" => " (quality)",
                        "xhigh" => " (max reasoning)",
                        _ => "",
                    };
                    ListItem::new(format!(" {}{}", e, note)).style(if i == self.sel_model {
                        Style::default().bg(Color::Blue).fg(Color::White)
                    } else {
                        Style::default()
                    })
                })
                .collect();
            f.render_widget(
                List::new(items).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("🎯 Reasoning Effort"),
                ),
                c[0],
            );
            f.render_widget(
                Paragraph::new("↑↓ Choose  Enter Confirm  Set for ALL phases")
                    .block(Block::default().borders(Borders::ALL)),
                c[1],
            );
        }
    }
    fn phases_screen(&self, f: &mut Frame, a: Rect) {
        let c = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(3)])
            .split(a);
        if self.edit_phase {
            // Editing a phase provider/model
            let items: Vec<ListItem> = PROVIDERS
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    let models = models_for_provider(p);
                    let model_list = models.join(", ");
                    ListItem::new(format!(" {}  — {}", p, model_list)).style(
                        if i == self.sel_provider {
                            Style::default().bg(Color::Blue).fg(Color::White)
                        } else {
                            Style::default()
                        },
                    )
                })
                .collect();
            let phase_name = &self.phase_models[self.phase_sel].0;
            f.render_widget(
                List::new(items).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(format!("📋 Phase: {} — Select Provider", phase_name)),
                ),
                c[0],
            );
            f.render_widget(
                Paragraph::new("↑↓ Choose  Enter Confirm  Esc Back")
                    .block(Block::default().borders(Borders::ALL)),
                c[1],
            );
        } else {
            let rows: Vec<Row> = SDD_PHASES
                .iter()
                .enumerate()
                .map(|(i, ph)| {
                    Row::new(vec![
                        if i == self.phase_sel { "→" } else { " " },
                        ph,
                        &self.phase_models[i].1,
                        &self.phase_models[i].2,
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
                .block(Block::default().borders(Borders::ALL).title(format!(
                    "📋 SDD Phases — Orchestrator: {}/{} ({})",
                    self.provider, self.model, self.effort
                ))),
                c[0],
            );
            f.render_widget(
                Paragraph::new(
                    "↑↓ Navigate  Enter Edit phase  Tab/Enter on last: Continue  Esc: Back",
                )
                .block(Block::default().borders(Borders::ALL)),
                c[1],
            );
        }
    }
    fn execute_screen(&self, f: &mut Frame, a: Rect) {
        let lines = vec![
            Line::from(Span::styled(
                "Review & Execute",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(format!(
                "  Agents: {}",
                self.agents
                    .iter()
                    .filter(|a| a.1)
                    .map(|a| a.0)
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
            Line::from(format!(
                "  Orchestrator: {}/{} ({})",
                self.provider, self.model, self.effort
            )),
            Line::from(format!(
                "  Profile: default ({} phases configured)",
                self.phase_models.len()
            )),
            Line::from(""),
            Line::from(Span::styled(
                "  Tab/Enter: Execute  Esc: Back",
                Style::default().fg(Color::Cyan),
            )),
        ];
        f.render_widget(
            Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title("▶ Execute")),
            a,
        );
    }
    fn done_screen(&self, f: &mut Frame, a: Rect) {
        let lines: Vec<Line> = self
            .logs
            .iter()
            .map(|l| {
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
            Paragraph::new(lines)
                .block(Block::default().borders(Borders::ALL).title("✅ Complete")),
            a,
        );
    }
}
