mod stateful_list;

use std::{path::PathBuf, process::Command, time::Duration};

use crossterm::event::{
    self,
    Event::Key,
    KeyCode::{Char, Down, Enter, Up},
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::config::{parser::SshConfigParser, SshHost};
use stateful_list::StatefulList;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
type Terminal = ratatui::Terminal<CrosstermBackend<std::io::Stderr>>;
type Frame<'a> = ratatui::Frame<'a, CrosstermBackend<std::io::Stderr>>;

#[derive(Default)]
pub struct AppState {
    hosts: StatefulList<SshHost>,
    host_file: PathBuf,
    should_quit: bool,
    should_connect: bool,
    should_edit: bool,
}

impl AppState {
    pub fn new(host_file: PathBuf) -> Result<Self> {
        let mut data = Self {
            host_file,
            ..Default::default()
        };

        data.reload_hosts()?;
        Ok(data)
    }

    pub fn connect(&mut self) -> Result<()> {
        if let Some(host) = self.hosts.current() {
            println!("Trying to connect to {}...", host.hostname);
            let _output = Command::new("ssh").args(host.get_command_line()).status()?;
            Ok(())
        } else {
            Ok(())
        }
    }

    pub fn run_editor(&mut self) -> Result<()> {
        let editor = std::env::var("EDITOR").unwrap_or_default();
        if editor.is_empty() {
            eprintln!("No editor found.");
            Ok(())
        } else {
            let _output = Command::new(editor)
                .arg(self.host_file.to_string_lossy().to_string())
                .status()?;
            Ok(())
        }
    }

    pub fn reload_hosts(&mut self) -> Result<()> {
        let parser = SshConfigParser::new();
        let config = parser.parse_from_path(self.host_file.clone())?;
        self.hosts.set_data(config.to_hosts());
        Ok(())
    }
}

pub fn run(host_file: PathBuf) -> Result<()> {
    let app_state = AppState::new(host_file)?;
    let app = App::new();
    app.run(app_state)
}

#[derive(Default)]
pub struct App;

impl App {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn run(&self, mut state: AppState) -> Result<()> {
        let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stderr()))?;
        self.startup(&mut terminal)?;

        state.hosts.select_first();

        loop {
            self.update(&mut state)?;

            terminal.draw(|f| self.ui(&mut state, f))?;

            if state.should_quit {
                break;
            }

            if state.should_connect {
                self.escape_raw_mode(&mut terminal, &mut state, |state| state.connect())?;

                state.should_connect = false;
            }

            if state.should_edit {
                self.escape_raw_mode(&mut terminal, &mut state, |state| {
                    state.run_editor()?;
                    state.reload_hosts()?;
                    state.hosts.select_first();
                    Ok(())
                })?;

                state.should_edit = false;
            }
        }

        Ok(())
    }

    pub fn ui(&self, state: &mut AppState, f: &mut Frame<'_>) {
        let arrow_size = 2;
        let padding = 2;
        let longest_ip_length = state
            .hosts
            .iter()
            .map(|h| h.hostname.len())
            .max()
            .unwrap_or(0);

        let top_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(1),
                    Constraint::Min(0),
                    Constraint::Length(3),
                ]
                .as_ref(),
            )
            .split(f.size());

        let middle_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Percentage(33),
                    Constraint::Percentage(33),
                    Constraint::Percentage(33),
                ]
                .as_ref(),
            )
            .split(top_chunks[1]);

        let middle_length =
            middle_chunks[1].width as usize - longest_ip_length - padding * 2 - arrow_size;

        // Write host aliases in left section
        let items: Vec<_> = state
            .hosts
            .iter()
            .map(|host| {
                ListItem::new(vec![Line::from(vec![
                    Span::from(format!("{0:<middle_length$}", host.alias.clone())),
                    Span::styled(
                        format!("{:>longest_ip_length$}", host.hostname),
                        Style::default().fg(Color::Gray),
                    ),
                ])])
            })
            .collect();

        // Title
        let title = Paragraph::new("sshh...")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        f.render_widget(title, top_chunks[0]);

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Aliases"))
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .highlight_symbol("> ");
        f.render_stateful_widget(list, middle_chunks[1], state.hosts.state_mut());

        // Help
        let help_text = Paragraph::new("Q to quit\nE to edit file\nENTER to connect")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        f.render_widget(help_text, top_chunks[2])
    }

    pub fn update(&self, state: &mut AppState) -> Result<()> {
        if event::poll(Duration::from_millis(250))? {
            if let Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Press {
                    match key.code {
                        Char('q') => state.should_quit = true,
                        Char('e') => state.should_edit = true,
                        Up => state.hosts.select_previous(),
                        Down => state.hosts.select_next(),
                        Enter => state.should_connect = true,
                        _ => (),
                    }
                }
            }
        }

        Ok(())
    }

    fn startup(&self, terminal: &mut Terminal) -> Result<()> {
        crossterm::terminal::enable_raw_mode()?;
        crossterm::execute!(
            terminal.backend_mut(),
            crossterm::terminal::EnterAlternateScreen
        )?;
        terminal.clear()?;

        Ok(())
    }

    fn shutdown(&self, terminal: &mut Terminal) -> Result<()> {
        crossterm::execute!(
            terminal.backend_mut(),
            crossterm::terminal::LeaveAlternateScreen
        )?;
        crossterm::terminal::disable_raw_mode()?;
        terminal.clear()?;
        terminal.show_cursor()?;

        Ok(())
    }

    fn escape_raw_mode(
        &self,
        terminal: &mut Terminal,
        state: &mut AppState,
        f: impl Fn(&mut AppState) -> Result<()>,
    ) -> Result<()> {
        self.shutdown(terminal)?;
        f(state)?;
        self.startup(terminal)?;

        Ok(())
    }
}
