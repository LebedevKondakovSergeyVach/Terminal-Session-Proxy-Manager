use crate::config::{AppConfig, I18n};
use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Terminal,
};
use std::io;

/// Runs the interactive TUI dashboard.
pub fn run_dashboard(config: &mut AppConfig, _i18n: &I18n) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(&mut terminal, config);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, config: &mut AppConfig) -> io::Result<()> {
    let mut list_state = ListState::default();
    
    // Sort profiles to have a deterministic order
    let mut profiles: Vec<_> = config.profiles.keys().cloned().collect();
    profiles.sort();

    // Find active index
    let active_idx = profiles.iter().position(|k| k == &config.active_profile).unwrap_or(0);
    list_state.select(Some(active_idx));

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Min(5),
                        Constraint::Length(3),
                    ]
                    .as_ref(),
                )
                .split(f.area());

            // Header
            let header = Paragraph::new(Line::from(vec![
                Span::styled("⚡ Terminal Session Proxy Manager ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled("- Dashboard", Style::default().fg(Color::White)),
            ]))
            .block(Block::default().borders(Borders::ALL));
            f.render_widget(header, chunks[0]);

            // Profiles List
            let items: Vec<ListItem> = profiles
                .iter()
                .map(|key| {
                    let profile = config.profiles.get(key).unwrap();
                    let content = format!(" {} | {}:{} ({})", profile.name, profile.host, profile.port, profile.protocol);
                    
                    let style = if key == &config.active_profile {
                        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White)
                    };

                    ListItem::new(Line::from(Span::styled(content, style)))
                })
                .collect();

            let list = List::new(items)
                .block(Block::default().title(" Profiles ").borders(Borders::ALL))
                .highlight_style(
                    Style::default()
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol(">> ");

            f.render_stateful_widget(list, chunks[1], &mut list_state);

            // Footer
            let footer = Paragraph::new(Line::from(vec![
                Span::styled(" Use ", Style::default().fg(Color::Gray)),
                Span::styled("↑↓", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(" to navigate | ", Style::default().fg(Color::Gray)),
                Span::styled("Enter", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::styled(" to switch | ", Style::default().fg(Color::Gray)),
                Span::styled("q", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::styled(" to quit", Style::default().fg(Color::Gray)),
            ]))
            .block(Block::default().borders(Borders::ALL));
            f.render_widget(footer, chunks[2]);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => {
                    return Ok(());
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    let i = match list_state.selected() {
                        Some(i) => {
                            if i >= profiles.len() - 1 {
                                0
                            } else {
                                i + 1
                            }
                        }
                        None => 0,
                    };
                    list_state.select(Some(i));
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    let i = match list_state.selected() {
                        Some(i) => {
                            if i == 0 {
                                profiles.len() - 1
                            } else {
                                i - 1
                            }
                        }
                        None => 0,
                    };
                    list_state.select(Some(i));
                }
                KeyCode::Enter => {
                    if let Some(i) = list_state.selected() {
                        let selected_key = &profiles[i];
                        config.active_profile = selected_key.clone();
                        let _ = config.save();
                    }
                }
                _ => {}
            }
        }
    }
}
