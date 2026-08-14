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
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Sparkline},
    Terminal,
};
use serde::Deserialize;
use std::io;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Default, Clone)]
struct DashState {
    ip: String,
    location: String,
    ping_ms: Option<u128>,
    ping_history: Vec<u64>,
    is_loading: bool,
    active_profile_key: String,
    is_local_listening: Option<bool>,
}

#[derive(Deserialize)]
struct GeoResponse {
    query: Option<String>,
    city: Option<String>,
    country: Option<String>,
}

enum DashAction {
    Quit,
    SelectBest,
    Import,
    EditConfig,
}

/// Runs the interactive TUI dashboard.
pub async fn run_dashboard(config: &mut AppConfig, _i18n: &I18n) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let state = Arc::new(Mutex::new(DashState {
        ip: "Loading...".to_string(),
        location: "Loading...".to_string(),
        ping_ms: None,
        ping_history: Vec::new(),
        is_loading: true,
        active_profile_key: config.active_profile.clone(),
        is_local_listening: None,
    }));

    let state_clone = state.clone();
    let config_clone = config.clone();

    // Background task to fetch IP and Latency
    tokio::spawn(async move {
        let mut last_key = String::new();
        loop {
            let current_key = {
                let s = state_clone.lock().unwrap();
                s.active_profile_key.clone()
            };

            if current_key != last_key {
                let mut s = state_clone.lock().unwrap();
                s.is_loading = true;
                s.ip = "Loading...".to_string();
                s.location = "Loading...".to_string();
                s.ping_ms = None;
                s.ping_history.clear();
                s.is_local_listening = None;
                last_key = current_key.clone();
            }

            let profile = config_clone.profiles.get(&current_key);
            if let Some(prof) = profile {
                let local_addr = format!("127.0.0.1:{}", prof.port);
                let listening = if let Ok(addr) = local_addr.parse() {
                    std::net::TcpStream::connect_timeout(&addr, Duration::from_millis(100)).is_ok()
                } else {
                    false
                };

                {
                    let mut s = state_clone.lock().unwrap();
                    if s.active_profile_key == current_key {
                        s.is_local_listening = Some(listening);
                    }
                }

                let proxy_url = format!("{}://{}:{}", prof.protocol, prof.host, prof.port);
                let mut builder = reqwest::Client::builder()
                    .timeout(Duration::from_secs(3))
                    .connect_timeout(Duration::from_secs(2));

                if let Ok(proxy) = reqwest::Proxy::all(&proxy_url) {
                    builder = builder.proxy(proxy);
                }

                if let Ok(client) = builder.build() {
                    let start = Instant::now();
                    if let Ok(resp) = client.get("http://ip-api.com/json").send().await {
                        let elapsed = start.elapsed().as_millis();
                        if let Ok(data) = resp.json::<GeoResponse>().await {
                            let mut s = state_clone.lock().unwrap();
                            if s.active_profile_key == current_key {
                                s.ip = data.query.unwrap_or_else(|| "Unknown".to_string());
                                let city = data.city.unwrap_or_default();
                                let country = data.country.unwrap_or_default();
                                s.location = if !city.is_empty() && !country.is_empty() {
                                    format!("{}, {}", city, country)
                                } else {
                                    country
                                };
                                s.ping_ms = Some(elapsed);

                                s.ping_history.push(elapsed as u64);
                                if s.ping_history.len() > 100 {
                                    s.ping_history.remove(0);
                                }

                                s.is_loading = false;
                            }
                        }
                    } else {
                        let mut s = state_clone.lock().unwrap();
                        if s.active_profile_key == current_key {
                            s.ip = "Offline".to_string();
                            s.location = "Offline".to_string();
                            s.ping_ms = None;

                            s.ping_history.push(0); // 0 implies timeout
                            if s.ping_history.len() > 100 {
                                s.ping_history.remove(0);
                            }

                            s.is_loading = false;
                        }
                    }
                }
            }
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    });

    let res = run_app(&mut terminal, config, state);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    match res {
        Ok(DashAction::SelectBest) => {
            crate::cmd::profile::select_best_profile(config, _i18n).await?;
        }
        Ok(DashAction::Import) => {
            let url: String = dialoguer::Input::new()
                .with_prompt("Enter Subscription URL/Path")
                .interact_text()?;
            crate::cmd::import_cmd::import_profiles(config, _i18n, &url).await?;
        }
        Ok(DashAction::EditConfig) => {
            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());
            let path = crate::config::AppConfig::get_config_path();
            let mut child = std::process::Command::new(editor)
                .arg(path)
                .spawn()
                .expect("Failed to start editor");
            let _ = child.wait();

            // Reload config if changed
            *config = crate::config::AppConfig::load();
            println!("Config reloaded successfully.");
        }
        Err(err) => {
            println!("{:?}", err);
        }
        _ => {}
    }

    Ok(())
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    config: &mut AppConfig,
    state: Arc<Mutex<DashState>>,
) -> io::Result<DashAction> {
    let mut list_state = ListState::default();

    let mut profiles: Vec<_> = config.profiles.keys().cloned().collect();
    profiles.sort();

    let active_idx = profiles
        .iter()
        .position(|k| k == &config.active_profile)
        .unwrap_or(0);
    list_state.select(Some(active_idx));

    loop {
        let current_state = { state.lock().unwrap().clone() };

        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Length(7),
                        Constraint::Min(5),
                        Constraint::Length(3),
                    ]
                    .as_ref(),
                )
                .split(f.area());

            let middle_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
                .split(chunks[1]);

            // Header
            let header = Paragraph::new(Line::from(vec![
                Span::styled(
                    "⚡ Terminal Session Proxy Manager ",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("- Dashboard", Style::default().fg(Color::White)),
            ]))
            .block(Block::default().borders(Borders::ALL));
            f.render_widget(header, chunks[0]);

            // Live Info Panel
            let ping_str = match current_state.ping_ms {
                Some(ms) if ms < 200 => Span::styled(
                    format!("{}ms", ms),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Some(ms) if ms < 500 => Span::styled(
                    format!("{}ms", ms),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Some(ms) => Span::styled(
                    format!("{}ms", ms),
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                None => Span::styled("Offline/Timeout", Style::default().fg(Color::Red)),
            };

            let ip_style = if current_state.is_loading {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            };

            let port_str = match current_state.is_local_listening {
                Some(true) => Span::styled(
                    "Active 🟢",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Some(false) => Span::styled(
                    "Dead 🔴 (Backend not running)",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                None => Span::styled("Checking...", Style::default().fg(Color::DarkGray)),
            };

            let info_text = vec![
                Line::from(vec![]),
                Line::from(vec![
                    Span::styled(" External IP: ", Style::default().fg(Color::White)),
                    Span::styled(&current_state.ip, ip_style),
                ]),
                Line::from(vec![
                    Span::styled(" Location:    ", Style::default().fg(Color::White)),
                    Span::styled(&current_state.location, ip_style),
                ]),
                Line::from(vec![
                    Span::styled(" Latency:     ", Style::default().fg(Color::White)),
                    ping_str,
                ]),
                Line::from(vec![
                    Span::styled(" Local Port:  ", Style::default().fg(Color::White)),
                    port_str,
                ]),
            ];

            let info_panel = Paragraph::new(info_text).block(
                Block::default()
                    .title(" Live Monitor ")
                    .borders(Borders::ALL),
            );
            f.render_widget(info_panel, middle_chunks[0]);

            // Sparkline Graph
            let sparkline = Sparkline::default()
                .block(
                    Block::default()
                        .title(" Ping History (ms) ")
                        .borders(Borders::ALL),
                )
                .data(&current_state.ping_history)
                .style(Style::default().fg(Color::Green));
            f.render_widget(sparkline, middle_chunks[1]);

            // Profiles List
            let items: Vec<ListItem> = profiles
                .iter()
                .map(|key| {
                    let profile = config.profiles.get(key).unwrap();
                    let content = format!(
                        " {} | {}:{} ({})",
                        profile.name, profile.host, profile.port, profile.protocol
                    );

                    let style = if key == &config.active_profile {
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD)
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

            f.render_stateful_widget(list, chunks[2], &mut list_state);

            // Footer
            let footer = Paragraph::new(Line::from(vec![
                Span::styled(" Use ", Style::default().fg(Color::Gray)),
                Span::styled(
                    "↑↓",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" nav | ", Style::default().fg(Color::Gray)),
                Span::styled(
                    "Enter",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" switch | ", Style::default().fg(Color::Gray)),
                Span::styled(
                    "b",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" best | ", Style::default().fg(Color::Gray)),
                Span::styled(
                    "i",
                    Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" import | ", Style::default().fg(Color::Gray)),
                Span::styled(
                    "e",
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" edit config | ", Style::default().fg(Color::Gray)),
                Span::styled(
                    "q",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::styled(" quit", Style::default().fg(Color::Gray)),
            ]))
            .block(Block::default().borders(Borders::ALL));
            f.render_widget(footer, chunks[3]);
        })?;

        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        return Ok(DashAction::Quit);
                    }
                    KeyCode::Char('b') => {
                        return Ok(DashAction::SelectBest);
                    }
                    KeyCode::Char('i') => {
                        return Ok(DashAction::Import);
                    }
                    KeyCode::Char('e') => {
                        return Ok(DashAction::EditConfig);
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

                            let mut s = state.lock().unwrap();
                            s.active_profile_key = selected_key.clone();
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}
