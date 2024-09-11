mod stateful_list;

use std::{path::PathBuf, process::Command, sync::mpsc::channel, time::Duration};

use crossterm::event::{
    self,
    Event::Key,
    KeyCode::{Backspace, Char, Down, Enter, Esc, Up},
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::config::{parser::SshConfigParser, SshHost};
use stateful_list::StatefulList;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
type Terminal = ratatui::Terminal<CrosstermBackend<std::io::Stdout>>;

#[derive(Default)]
pub struct AppState {
    all_hosts: Vec<SshHost>,
    filtered_hosts: StatefulList<SshHost>,
    host_file: PathBuf,
    should_quit: bool,
    should_connect: bool,
    should_edit: bool,
    search_mode: bool,
    search_query: String,
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
        if let Some(host) = self.filtered_hosts.current() {
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
        self.all_hosts = config.to_hosts();
        self.update_filtered_hosts();
        Ok(())
    }

    pub fn update_filtered_hosts(&mut self) {
        let filtered = self
            .all_hosts
            .iter()
            .filter(|h| {
                h.alias
                    .to_lowercase()
                    .contains(&self.search_query.to_lowercase())
            })
            .cloned()
            .collect::<Vec<_>>();
        self.filtered_hosts.set_data(filtered);
        self.filtered_hosts.select_first();
    }

    pub fn longest_ip_length(&self) -> usize {
        self.filtered_hosts
            .iter()
            .map(|h| h.hostname.len())
            .max()
            .unwrap_or(0)
    }

    pub fn reset_search(&mut self) {
        self.search_mode = false;
        self.search_query.clear();
        self.update_filtered_hosts();
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
        let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))?;
        self.startup(&mut terminal)?;

        state.filtered_hosts.select_first();

        let (tx, rx) = channel();
        ctrlc::set_handler(move || tx.send(()).expect("Could not send signal"))
            .expect("Error setting SIGINT handler");

        loop {
            self.update(&mut state)?;

            terminal.draw(|f| self.ui(&mut state, f))?;

            if state.should_quit {
                self.shutdown(&mut terminal)?;
                break;
            }

            if state.should_connect {
                self.escape_raw_mode(&mut terminal, &mut state, |state| state.connect())?;
                state.reset_search();
                state.should_connect = false;
            }

            if state.should_edit {
                self.escape_raw_mode(&mut terminal, &mut state, |state| {
                    state.run_editor()?;
                    state.reload_hosts()?;
                    state.filtered_hosts.select_first();
                    Ok(())
                })?;
                state.reset_search();
                state.should_edit = false;
            }

            if rx.try_recv().is_ok() {
                // CTRL+C received, try again!
                continue;
            }
        }

        Ok(())
    }

    pub fn ui(&self, state: &mut AppState, f: &mut Frame<'_>) {
        let arrow_size = 2;
        let padding = 2;
        let longest_ip_length = state.longest_ip_length();

        let top_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(1),
                    Constraint::Min(0),
                    Constraint::Length(1),
                    Constraint::Length(4),
                ]
                .as_ref(),
            )
            .split(f.area());

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
            .filtered_hosts
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
        let title = Paragraph::new(format!("sshh {}", env!("CARGO_PKG_VERSION")))
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        f.render_widget(title, top_chunks[0]);

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Aliases"))
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .highlight_symbol("> ");
        f.render_stateful_widget(list, middle_chunks[1], state.filtered_hosts.state_mut());

        // Search
        if !state.search_query.is_empty() || state.search_mode {
            let search =
                Paragraph::new(format!("/{}", state.search_query)).alignment(Alignment::Center);
            f.render_widget(search, top_chunks[2]);
        }

        // Help
        let help_text_content = if state.search_mode {
            "Arrow up/down to select    ENTER to connect\nType characters to search    ESC to cancel search"
        } else {
            "Arrow up/down to select    ENTER to connect\nQ to quit    E to edit file (using EDITOR var)\n/ to search"
        };

        let help_text = Paragraph::new(help_text_content)
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        f.render_widget(help_text, top_chunks[3])
    }

    pub fn update(&self, state: &mut AppState) -> Result<()> {
        if event::poll(Duration::from_millis(250))? {
            if let Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Press {
                    if state.search_mode {
                        match key.code {
                            Esc => {
                                state.reset_search();
                            }
                            Char(c) => {
                                state.search_query.push(c);
                                state.update_filtered_hosts();
                            }
                            Backspace => {
                                if !state.search_query.is_empty() {
                                    state.search_query.pop();
                                } else {
                                    state.search_mode = false;
                                }
                                state.update_filtered_hosts();
                            }
                            _ => (),
                        }
                    } else {
                        match key.code {
                            Char('q') => state.should_quit = true,
                            Char('e') => state.should_edit = true,
                            Char('/') => state.search_mode = true,
                            _ => (),
                        }
                    }

                    match key.code {
                        Up => state.filtered_hosts.select_previous(),
                        Down => state.filtered_hosts.select_next(),
                        Enter => {
                            if state.filtered_hosts.current().is_some() {
                                state.should_connect = true;
                            }
                        }
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
