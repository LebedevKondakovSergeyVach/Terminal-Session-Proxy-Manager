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
    is_benchmarking_all: bool,
    benchmark_results: Option<Vec<(String, u128)>>,
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
        is_benchmarking_all: false,
        benchmark_results: None,
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

    let mut tab_index = 0; // 0 = Profiles, 1 = Config
    let mut config_scroll: u16 = 0;

    loop {
        let current_state = { state.lock().unwrap().clone() };

        // Sort dynamically if benchmark results exist
        if let Some(ref results) = current_state.benchmark_results {
            profiles.sort_by_key(|k| {
                results
                    .iter()
                    .find(|(key, _)| key == k)
                    .map(|(_, p)| *p)
                    .unwrap_or(u128::MAX)
            });
        }

        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Length(3), // Header & Tabs
                        Constraint::Min(5),    // Main Content
                        Constraint::Length(3), // Footer
                    ]
                    .as_ref(),
                )
                .split(f.area());

            let header_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
                .split(chunks[0]);

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
            f.render_widget(header, header_chunks[0]);

            // Tabs
            let titles: Vec<Line> = vec![" [1] Profiles ", " [2] Config JSON "]
                .into_iter()
                .map(Line::from)
                .collect();
            let tabs = ratatui::widgets::Tabs::new(titles)
                .block(Block::default().borders(Borders::ALL))
                .select(tab_index)
                .highlight_style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                );
            f.render_widget(tabs, header_chunks[1]);

            if tab_index == 0 {
                let main_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(7), Constraint::Min(5)])
                    .split(chunks[1]);

                let middle_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
                    .split(main_chunks[0]);

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

                let title = if current_state.is_benchmarking_all {
                    " Profiles [Benchmarking...] "
                } else {
                    " Profiles "
                };

                let list = List::new(items)
                    .block(Block::default().title(title).borders(Borders::ALL))
                    .highlight_style(
                        Style::default()
                            .bg(Color::DarkGray)
                            .add_modifier(Modifier::BOLD),
                    )
                    .highlight_symbol(">> ");

                f.render_stateful_widget(list, main_chunks[1], &mut list_state);
            } else {
                // Config Tab
                let config_json = serde_json::to_string_pretty(config).unwrap_or_default();
                let paragraph = Paragraph::new(config_json)
                    .block(
                        Block::default()
                            .title(" Config JSON ")
                            .borders(Borders::ALL),
                    )
                    .scroll((config_scroll, 0));
                f.render_widget(paragraph, chunks[1]);
            }

            // Footer
            let footer_text = if tab_index == 0 {
                vec![
                    Span::styled(" Use ", Style::default().fg(Color::Gray)),
                    Span::styled(
                        "↑↓",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(" nav | ", Style::default().fg(Color::Gray)),
                    Span::styled(
                        "Space",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(" switch | ", Style::default().fg(Color::Gray)),
                    Span::styled(
                        "Enter",
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(" use | ", Style::default().fg(Color::Gray)),
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
                    Span::styled(" edit | ", Style::default().fg(Color::Gray)),
                    Span::styled(
                        "s",
                        Style::default()
                            .fg(Color::LightGreen)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(" bench+sort | ", Style::default().fg(Color::Gray)),
                    Span::styled(
                        "1/2",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(" tabs | ", Style::default().fg(Color::Gray)),
                    Span::styled(
                        "q",
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(" quit", Style::default().fg(Color::Gray)),
                ]
            } else {
                vec![
                    Span::styled(" Use ", Style::default().fg(Color::Gray)),
                    Span::styled(
                        "↑↓",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(" scroll | ", Style::default().fg(Color::Gray)),
                    Span::styled(
                        "1/2",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(" tabs | ", Style::default().fg(Color::Gray)),
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
                ]
            };

            let footer = Paragraph::new(Line::from(footer_text))
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(footer, chunks[2]);
        })?;

        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        return Ok(DashAction::Quit);
                    }
                    KeyCode::Char('1') => {
                        tab_index = 0;
                    }
                    KeyCode::Char('2') => {
                        tab_index = 1;
                    }
                    KeyCode::Char('b') if tab_index == 0 => {
                        return Ok(DashAction::SelectBest);
                    }
                    KeyCode::Char('i') if tab_index == 0 => {
                        return Ok(DashAction::Import);
                    }
                    KeyCode::Char('e') => {
                        return Ok(DashAction::EditConfig);
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if tab_index == 0 {
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
                        } else {
                            config_scroll = config_scroll.saturating_add(1);
                        }
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        if tab_index == 0 {
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
                        } else {
                            config_scroll = config_scroll.saturating_sub(1);
                        }
                    }
                    KeyCode::Char('s') if tab_index == 0 => {
                        let mut s = state.lock().unwrap();
                        if !s.is_benchmarking_all {
                            s.is_benchmarking_all = true;
                            s.benchmark_results = None;
                            let state_clone = state.clone();
                            let config_clone = config.clone();

                            tokio::spawn(async move {
                                let mut results = Vec::new();
                                let timeout = Duration::from_millis(3000);

                                for (key, prof) in &config_clone.profiles {
                                    let proxy_url =
                                        format!("{}://{}:{}", prof.protocol, prof.host, prof.port);
                                    let mut tasks = Vec::new();

                                    for ep in &config_clone.ping_targets {
                                        let p_url = proxy_url.clone();
                                        let target_url = ep.url.clone();

                                        tasks.push(tokio::spawn(async move {
                                            let mut builder = reqwest::Client::builder()
                                                .timeout(timeout)
                                                .connect_timeout(timeout);
                                            if let Ok(proxy) = reqwest::Proxy::all(&p_url) {
                                                builder = builder.proxy(proxy);
                                            }
                                            let client = match builder.build() {
                                                Ok(c) => c,
                                                Err(_) => return None,
                                            };
                                            let start = Instant::now();
                                            if client.get(&target_url).send().await.is_ok() {
                                                Some(start.elapsed().as_millis())
                                            } else {
                                                None
                                            }
                                        }));
                                    }

                                    let task_results = futures::future::join_all(tasks).await;
                                    let mut total_ms = 0u128;
                                    let mut success_count = 0usize;

                                    for ms in task_results.into_iter().flatten().flatten() {
                                        total_ms += ms;
                                        success_count += 1;
                                    }

                                    let avg_ms = if success_count > 0 {
                                        total_ms / (success_count as u128)
                                    } else {
                                        u128::MAX // timeout
                                    };

                                    results.push((key.clone(), avg_ms));
                                }

                                let mut s = state_clone.lock().unwrap();
                                s.benchmark_results = Some(results);
                                s.is_benchmarking_all = false;
                            });
                        }
                    }
                    KeyCode::Char(' ') if tab_index == 0 => {
                        if let Some(i) = list_state.selected() {
                            let selected_key = &profiles[i];
                            config.active_profile = selected_key.clone();
                            let _ = config.save();

                            let mut s = state.lock().unwrap();
                            s.active_profile_key = selected_key.clone();
                        }
                    }
                    KeyCode::Enter if tab_index == 0 => {
                        if let Some(i) = list_state.selected() {
                            let selected_key = &profiles[i];
                            config.active_profile = selected_key.clone();
                            let _ = config.save();

                            let mut s = state.lock().unwrap();
                            s.active_profile_key = selected_key.clone();

                            // Export to proxy-cli-eval so parent shell can eval it
                            if let Some(prof) = config.profiles.get(selected_key) {
                                if let Some(mut path) = dirs::home_dir() {
                                    use std::io::Write;
                                    let mut log_path = path.clone();
                                    log_path.push(".proxy-cli-debug.log");

                                    let mut debug_enabled_path = path.clone();
                                    debug_enabled_path.push(".proxy-cli-debug-enabled");
                                    let is_debug = debug_enabled_path.exists();

                                    let mut log_f = if is_debug {
                                        std::fs::OpenOptions::new()
                                            .create(true)
                                            .append(true)
                                            .open(&log_path)
                                            .ok()
                                    } else {
                                        None
                                    };

                                    if let Some(ref mut l) = log_f {
                                        let _ = writeln!(
                                            l,
                                            "[dash] Processing Enter. Selected profile: {}",
                                            selected_key
                                        );
                                    }

                                    path.push(".proxy-cli-eval");
                                    match std::fs::File::create(&path) {
                                        Ok(mut f) => {
                                            let url = format!(
                                                "{}://{}:{}",
                                                prof.protocol, prof.host, prof.port
                                            );
                                            let content = format!("export HTTP_PROXY={0}; export HTTPS_PROXY={0}; export ALL_PROXY={0};", url);
                                            if let Err(e) = write!(f, "{}", content) {
                                                if let Some(ref mut l) = log_f {
                                                    let _ = writeln!(
                                                        l,
                                                        "[dash] Failed to write eval file: {}",
                                                        e
                                                    );
                                                }
                                            } else {
                                                if let Some(ref mut l) = log_f {
                                                    let _ = writeln!(l, "[dash] Successfully wrote eval file to {:?} with content: {}", path, content);
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            if let Some(ref mut l) = log_f {
                                                let _ = writeln!(
                                                    l,
                                                    "[dash] Failed to create eval file: {}",
                                                    e
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        return Ok(DashAction::Quit);
                    }
                    _ => {}
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dash_state_default() {
        let state = DashState::default();
        assert_eq!(state.ip, "");
        assert_eq!(state.location, "");
        assert_eq!(state.ping_ms, None);
        assert!(!state.is_benchmarking_all);
        assert!(state.benchmark_results.is_none());
    }
}
