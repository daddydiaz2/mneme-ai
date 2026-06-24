use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, List, ListItem, Paragraph, Row, Table},
    Terminal,
};
use std::io;

use crate::profile::{ProfileStore, SddProfile, SDD_PHASES};

type Term = Terminal<ratatui::backend::CrosstermBackend<io::Stdout>>;

pub fn run_tui() -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let store = ProfileStore::new();
    store.init()?;

    let mut app = App::new(store);
    let res = app.run(&mut terminal);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    res
}

enum TuiMode {
    Dashboard,
    ProfileList,
    ProfileDetail,
    CreateProfile,
}

struct App {
    store: ProfileStore,
    profiles: Vec<SddProfile>,
    selected: usize,
    mode: TuiMode,
    status_message: String,
    new_name: String,
    new_provider: String,
    new_model: String,
    create_step: usize,
}

impl App {
    fn new(store: ProfileStore) -> Self {
        let profiles = store.list().unwrap_or_default();
        Self {
            store,
            profiles,
            selected: 0,
            mode: TuiMode::Dashboard,
            status_message: String::new(),
            new_name: String::new(),
            new_provider: String::new(),
            new_model: String::new(),
            create_step: 0,
        }
    }

    fn run(&mut self, terminal: &mut Term) -> anyhow::Result<()> {
        loop {
            let area = terminal.size()?;
            terminal.draw(|f| self.render(f, area))?;

            if let Event::Key(key) = event::read()? {
                let handled = match self.mode {
                    TuiMode::Dashboard => self.handle_dashboard(key.code),
                    TuiMode::ProfileList => self.handle_profile_list(key.code),
                    TuiMode::ProfileDetail => self.handle_profile_detail(key.code),
                    TuiMode::CreateProfile => self.handle_create_profile(key.code),
                };
                if !handled {
                    break;
                }
            }
        }
        Ok(())
    }

    fn handle_dashboard(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Char('1') => {
                self.mode = TuiMode::ProfileList;
                true
            }
            KeyCode::Char('q') | KeyCode::Esc => false,
            _ => true,
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
                    self.mode = TuiMode::ProfileDetail;
                }
                true
            }
            KeyCode::Char('n') => {
                self.mode = TuiMode::CreateProfile;
                self.create_step = 0;
                self.new_name.clear();
                self.new_provider.clear();
                self.new_model.clear();
                true
            }
            KeyCode::Char('d') => {
                let name_to_delete = self.profiles.get(self.selected).map(|p| p.name.clone());
                if let Some(ref name) = name_to_delete {
                    if name != "default" {
                        self.store.delete(name).ok();
                        self.profiles = self.store.list().unwrap_or_default();
                        self.status_message = format!("Deleted: {}", name);
                    } else {
                        self.status_message = "Cannot delete default".to_string();
                    }
                }
                true
            }
            KeyCode::Esc => {
                self.mode = TuiMode::Dashboard;
                true
            }
            KeyCode::Char('q') => false,
            _ => true,
        }
    }

    fn handle_profile_detail(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Esc => {
                self.mode = TuiMode::ProfileList;
                true
            }
            KeyCode::Char('q') => false,
            _ => true,
        }
    }

    fn handle_create_profile(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Char(c) if c != '\n' && c != '\t' => {
                match self.create_step {
                    0 => self.new_name.push(c),
                    1 => self.new_provider.push(c),
                    2 => self.new_model.push(c),
                    _ => {}
                }
                true
            }
            KeyCode::Backspace => {
                match self.create_step {
                    0 => {
                        self.new_name.pop();
                    }
                    1 => {
                        self.new_provider.pop();
                    }
                    2 => {
                        self.new_model.pop();
                    }
                    _ => {}
                }
                true
            }
            KeyCode::Tab | KeyCode::Enter => {
                if self.create_step < 2 {
                    self.create_step += 1;
                } else if !self.new_name.is_empty() {
                    let profile = SddProfile {
                        name: self.new_name.clone().to_lowercase().replace(' ', "-"),
                        orchestrator: crate::profile::ModelAssignment {
                            provider: if self.new_provider.is_empty() {
                                "opencode".to_string()
                            } else {
                                self.new_provider.clone()
                            },
                            model: if self.new_model.is_empty() {
                                "default".to_string()
                            } else {
                                self.new_model.clone()
                            },
                            reasoning_effort: None,
                        },
                        phases: std::collections::HashMap::new(),
                    };
                    self.store.save(&profile).ok();
                    self.profiles = self.store.list().unwrap_or_default();
                    self.status_message = format!("Created: {}", profile.name);
                    self.mode = TuiMode::ProfileList;
                }
                true
            }
            KeyCode::Esc => {
                self.mode = TuiMode::ProfileList;
                true
            }
            KeyCode::Char('q') => false,
            _ => true,
        }
    }

    fn render(&self, f: &mut ratatui::Frame, area: Rect) {
        match self.mode {
            TuiMode::Dashboard => self.render_dashboard(f, area),
            TuiMode::ProfileList => self.render_profile_list(f, area),
            TuiMode::ProfileDetail => self.render_profile_detail(f, area),
            TuiMode::CreateProfile => self.render_create_profile(f, area),
        }
    }

    fn render_dashboard(&self, f: &mut ratatui::Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(5),
                Constraint::Min(5),
            ])
            .split(area);

        let title = Paragraph::new("mneme-ai v0.1.0 — Ecosystem Configurator")
            .style(Style::default().add_modifier(Modifier::BOLD))
            .block(Block::default().borders(Borders::ALL).title("🧠 mneme-ai"));
        f.render_widget(title, chunks[0]);

        let info = Paragraph::new(vec![
            Line::from(format!("SDD Profiles: {}", self.profiles.len())),
            Line::from("Supported Agents: 11"),
            Line::from(""),
            Line::from(Span::styled(
                "Press 1: Manage Profiles  |  q: Quit",
                Style::default().fg(Color::Cyan),
            )),
        ])
        .block(Block::default().borders(Borders::ALL).title("Status"));
        f.render_widget(info, chunks[1]);

        let help = Paragraph::new(vec![
            Line::from("Commands:"),
            Line::from("  mneme-ai init              — Initialize config"),
            Line::from("  mneme-ai install <agent>   — Configure agent"),
            Line::from("  mneme-ai doctor            — Health check"),
            Line::from("  mneme-ai tui               — This TUI"),
            Line::from("  mneme-ai profile list      — List profiles"),
        ])
        .block(Block::default().borders(Borders::ALL).title("Help"));
        f.render_widget(help, chunks[2]);
    }

    fn render_profile_list(&self, f: &mut ratatui::Frame, area: Rect) {
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
                ListItem::new(format!(
                    " {}  [{}/{}] {} phases",
                    p.name,
                    p.orchestrator.provider,
                    p.orchestrator.model,
                    p.phases.len()
                ))
                .style(style)
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("SDD Profiles"))
            .highlight_style(Style::default().add_modifier(Modifier::BOLD));
        f.render_widget(list, chunks[0]);

        let msg = if !self.status_message.is_empty() {
            format!("  |  {}", self.status_message)
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

    fn render_profile_detail(&self, f: &mut ratatui::Frame, area: Rect) {
        if let Some(profile) = self.profiles.get(self.selected) {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(3),
                    Constraint::Length(3),
                ])
                .split(area);

            let header = Paragraph::new(format!(
                "Profile: {}  |  Orchestrator: {}/{}",
                profile.name, profile.orchestrator.provider, profile.orchestrator.model
            ))
            .style(Style::default().add_modifier(Modifier::BOLD))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("📋 Profile Detail"),
            );
            f.render_widget(header, chunks[0]);

            let widths = [
                Constraint::Length(20),
                Constraint::Length(15),
                Constraint::Length(25),
            ];
            let mut rows = vec![Row::new(vec!["Phase", "Provider", "Model"])
                .style(Style::default().add_modifier(Modifier::BOLD))];
            for phase in SDD_PHASES {
                let assignment = profile.phases.get(*phase).unwrap_or(&profile.orchestrator);
                let prov: &str = &assignment.provider;
                let mdl: &str = &assignment.model;
                rows.push(Row::new(vec![*phase, prov, mdl]));
            }
            let table = Table::new(rows, widths).block(Block::default().borders(Borders::ALL));
            f.render_widget(table, chunks[1]);

            let help = Paragraph::new("Esc: Back").block(Block::default().borders(Borders::ALL));
            f.render_widget(help, chunks[2]);
        }
    }

    fn render_create_profile(&self, f: &mut ratatui::Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(3),
                Constraint::Length(3),
            ])
            .split(area);

        let header = Paragraph::new("Create New SDD Profile")
            .style(Style::default().add_modifier(Modifier::BOLD))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(header, chunks[0]);

        let step_indicator = |s: usize| if self.create_step == s { "◄" } else { "✓" };
        let fields = vec![
            Line::from(format!(
                "Profile name [a-z, hyphens]: {} {}",
                self.new_name,
                step_indicator(0)
            )),
            Line::from(format!(
                "Provider [default: opencode]: {} {}",
                self.new_provider,
                step_indicator(1)
            )),
            Line::from(format!(
                "Model [default: default]: {} {}",
                self.new_model,
                step_indicator(2)
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Tab/Enter: Next  |  Esc: Cancel",
                Style::default().fg(Color::Cyan),
            )),
        ];
        let text = Paragraph::new(fields).block(Block::default().borders(Borders::ALL));
        f.render_widget(text, chunks[1]);
    }
}
