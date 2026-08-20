use crate::config::{AppConfig, I18n};
use crate::proxy_env;
use crate::shell_handoff;
use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Sparkline, Tabs},
};
use serde::Deserialize;
use std::io;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// How many latency samples the sparkline keeps.
const PING_HISTORY_LEN: usize = 100;
/// Delay between background refreshes of IP, location and latency.
const REFRESH_INTERVAL: Duration = Duration::from_secs(2);
/// How long the event loop blocks before redrawing.
const EVENT_POLL: Duration = Duration::from_millis(250);

type Backend = CrosstermBackend<io::Stdout>;

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
    ip: Option<String>,
    query: Option<String>,
    city: Option<String>,
    country: Option<String>,
}

/// What the user asked for by leaving the dashboard.
///
/// Actions that need a normal terminal (an editor, an interactive prompt) are
/// returned rather than performed in place, so they run after the alternate
/// screen has been torn down.
enum DashAction {
    Quit,
    /// Quit and export the selected profile to the parent shell.
    Apply(String),
    SelectBest,
    Import,
    EditConfig,
}

/// Restores the terminal when it goes out of scope, however that happens.
///
/// The previous implementation only restored on the success path, so any error
/// or panic inside the event loop left the user's terminal in raw mode on the
/// alternate screen — recoverable only by blindly typing `reset`.
struct TerminalGuard {
    terminal: Terminal<Backend>,
}

impl TerminalGuard {
    fn enter() -> Result<Self> {
        enable_raw_mode().context("could not put the terminal into raw mode")?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen).context("could not enter the alternate screen")?;
        let terminal =
            Terminal::new(CrosstermBackend::new(stdout)).context("could not initialise the TUI")?;

        Ok(Self { terminal })
    }

    fn restore() {
        // Best-effort and idempotent: this also runs from the panic hook, where
        // returning an error is not an option.
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        Self::restore();
        let _ = self.terminal.show_cursor();
    }
}

/// Runs the interactive TUI dashboard.
///
/// # Errors
/// Returns an error if the terminal cannot be initialised or a follow-up
/// action fails.
pub async fn run_dashboard(config: &mut AppConfig, i18n: &I18n) -> Result<()> {
    // A panic inside ratatui's draw closure would otherwise print its message
    // into the alternate screen and vanish, leaving a wedged terminal.
    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        TerminalGuard::restore();
        previous_hook(info);
    }));

    let state = Arc::new(Mutex::new(DashState {
        ip: i18n.t("dash_loading").to_string(),
        location: i18n.t("dash_loading").to_string(),
        is_loading: true,
        active_profile_key: config.active_profile.clone(),
        ..DashState::default()
    }));

    let refresher = tokio::spawn(refresh_loop(
        Arc::clone(&state),
        config.clone(),
        i18n.clone(),
    ));

    let action = {
        let mut guard = TerminalGuard::enter()?;
        let action = run_app(&mut guard.terminal, config, &Arc::clone(&state), i18n);
        // `guard` drops here, restoring the terminal before anything below
        // writes to stdout or opens an editor.
        action
    }?;

    refresher.abort();
    let _ = std::panic::take_hook();

    handle_action(action, config, i18n).await
}

/// Performs whatever the user selected, now that the terminal is usable again.
async fn handle_action(action: DashAction, config: &mut AppConfig, i18n: &I18n) -> Result<()> {
    match action {
        DashAction::Quit => {}
        DashAction::Apply(key) => {
            // Reuse the shared generator so the dashboard exports exactly what
            // `env on` does — it used to emit a smaller, different set.
            if let Some(statements) = proxy_env::export_statements(config) {
                shell_handoff::write_exports(&statements)?;
                shell_handoff::log_debug(&format!("dash applied profile '{key}'"));
            }
        }
        DashAction::SelectBest => crate::cmd::profile::select_best_profile(config, i18n).await?,
        DashAction::Import => {
            let source: String = dialoguer::Input::new()
                .with_prompt(i18n.t("dash_import_prompt"))
                .interact_text()?;
            crate::cmd::import_cmd::import_profiles(config, i18n, &source).await?;
        }
        DashAction::EditConfig => {
            let editor = std::env::var("VISUAL")
                .or_else(|_| std::env::var("EDITOR"))
                .unwrap_or_else(|_| "vi".to_string());
            let path = AppConfig::get_config_path();

            let status = std::process::Command::new(&editor)
                .arg(&path)
                .status()
                .with_context(|| format!("could not start editor '{editor}'"))?;

            if !status.success() {
                // Reloading after a crashed editor risks reading a truncated
                // file, so keep what is already in memory.
                eprintln!("{}", i18n.t("dash_editor_failed"));
                return Ok(());
            }

            // Report a broken edit instead of silently reverting to defaults.
            match AppConfig::load_from(&path) {
                Ok(Some(reloaded)) => {
                    *config = reloaded;
                    println!("{}", i18n.t("dash_config_reloaded"));
                }
                Ok(None) => {}
                Err(err) => eprintln!("{err}"),
            }
        }
    }
    Ok(())
}

/// Background task refreshing IP, geolocation, latency and local port state.
async fn refresh_loop(state: Arc<Mutex<DashState>>, config: AppConfig, i18n: I18n) {
    let mut last_key = String::new();

    loop {
        let current_key = with_state(&state, |s| s.active_profile_key.clone());

        if current_key != last_key {
            with_state(&state, |s| {
                s.is_loading = true;
                s.ip = i18n.t("dash_loading").to_string();
                s.location = i18n.t("dash_loading").to_string();
                s.ping_ms = None;
                s.ping_history.clear();
                s.is_local_listening = None;
            });
            last_key.clone_from(&current_key);
        }

        if let Some(profile) = config.profiles.get(&current_key) {
            let listening = format!("127.0.0.1:{}", profile.port)
                .parse()
                .is_ok_and(|addr| {
                    std::net::TcpStream::connect_timeout(&addr, Duration::from_millis(100)).is_ok()
                });

            with_state(&state, |s| {
                if s.active_profile_key == current_key {
                    s.is_local_listening = Some(listening);
                }
            });

            let proxy_url = format!(
                "{}://{}:{}",
                profile.protocol,
                profile.url_host(),
                profile.port
            );
            let geo_url = config
                .geo_apis
                .first()
                .cloned()
                .unwrap_or_else(|| "http://ip-api.com/json".to_string());

            let probe = fetch_geo(&proxy_url, &geo_url).await;

            with_state(&state, |s| {
                if s.active_profile_key != current_key {
                    return;
                }
                match probe {
                    Some((geo, elapsed)) => {
                        s.ip = geo
                            .query
                            .or(geo.ip)
                            .unwrap_or_else(|| i18n.t("unknown").to_string());
                        let city = geo.city.unwrap_or_default();
                        let country = geo.country.unwrap_or_default();
                        s.location = if city.is_empty() {
                            country
                        } else if country.is_empty() {
                            city
                        } else {
                            format!("{city}, {country}")
                        };
                        s.ping_ms = Some(elapsed);
                        push_history(
                            &mut s.ping_history,
                            u64::try_from(elapsed).unwrap_or(u64::MAX),
                        );
                    }
                    None => {
                        s.ip = i18n.t("dash_offline").to_string();
                        s.location = i18n.t("dash_offline").to_string();
                        s.ping_ms = None;
                        push_history(&mut s.ping_history, 0);
                    }
                }
                s.is_loading = false;
            });
        }

        tokio::time::sleep(REFRESH_INTERVAL).await;
    }
}

async fn fetch_geo(proxy_url: &str, geo_url: &str) -> Option<(GeoResponse, u128)> {
    // No direct-connection fallback: showing the machine's own IP in a panel
    // labelled with the proxy profile is worse than showing "offline".
    let proxy = reqwest::Proxy::all(proxy_url).ok()?;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .connect_timeout(Duration::from_secs(2))
        .proxy(proxy)
        .build()
        .ok()?;
    let start = Instant::now();
    let response = client.get(geo_url).send().await.ok()?;
    let elapsed = start.elapsed().as_millis();

    Some((response.json::<GeoResponse>().await.ok()?, elapsed))
}

/// Appends a sample, dropping the oldest once the window is full.
fn push_history(history: &mut Vec<u64>, sample: u64) {
    if history.len() >= PING_HISTORY_LEN {
        history.remove(0);
    }
    history.push(sample);
}

/// Runs a closure against the shared state, tolerating a poisoned lock.
///
/// A panic in one task must not turn every later state access into a second
/// panic that takes the whole dashboard down.
fn with_state<T>(state: &Arc<Mutex<DashState>>, f: impl FnOnce(&mut DashState) -> T) -> T {
    let mut guard = state
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    f(&mut guard)
}

/// Moves the selection by one, wrapping, on a list that may be empty.
///
/// Returns `None` for an empty list. The previous code computed
/// `profiles.len() - 1` unconditionally, which underflowed and panicked the
/// moment someone pressed an arrow key with no profiles configured.
fn step_selection(current: Option<usize>, len: usize, forward: bool) -> Option<usize> {
    if len == 0 {
        return None;
    }
    let current = current.unwrap_or(0).min(len - 1);
    Some(if forward {
        (current + 1) % len
    } else {
        (current + len - 1) % len
    })
}

fn run_app(
    terminal: &mut Terminal<Backend>,
    config: &mut AppConfig,
    state: &Arc<Mutex<DashState>>,
    i18n: &I18n,
) -> Result<DashAction> {
    let mut list_state = ListState::default();
    let mut profiles: Vec<String> = config.profiles.keys().cloned().collect();
    profiles.sort();

    list_state.select((!profiles.is_empty()).then(|| {
        profiles
            .iter()
            .position(|k| k == &config.active_profile)
            .unwrap_or(0)
    }));

    let mut tab_index = 0usize;
    let mut config_scroll: u16 = 0;

    loop {
        let snapshot = with_state(state, |s| s.clone());

        if let Some(ref results) = snapshot.benchmark_results {
            // Re-sorting moves rows under the cursor. Remember which profile
            // was selected and restore it afterwards, otherwise the highlight
            // silently jumps and Enter applies whatever landed on that index.
            let selected_key = list_state.selected().and_then(|i| profiles.get(i)).cloned();

            profiles.sort_by_key(|k| {
                results
                    .iter()
                    .find(|(key, _)| key == k)
                    .map_or(u128::MAX, |(_, ms)| *ms)
            });

            if let Some(key) = selected_key {
                list_state.select(profiles.iter().position(|k| *k == key));
            }
        }

        terminal.draw(|frame| {
            draw(
                frame,
                config,
                i18n,
                &snapshot,
                &profiles,
                &mut list_state,
                tab_index,
                config_scroll,
            );
        })?;

        if !event::poll(EVENT_POLL)? {
            continue;
        }
        let Event::Key(key) = event::read()? else {
            continue;
        };
        // Windows terminals deliver both press and release; acting on both
        // would move the selection two steps per keystroke.
        if key.kind != KeyEventKind::Press {
            continue;
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return Ok(DashAction::Quit),
            KeyCode::Char('1') => tab_index = 0,
            KeyCode::Char('2') => tab_index = 1,
            KeyCode::Char('b') if tab_index == 0 => return Ok(DashAction::SelectBest),
            KeyCode::Char('i') if tab_index == 0 => return Ok(DashAction::Import),
            KeyCode::Char('e') => return Ok(DashAction::EditConfig),

            KeyCode::Down | KeyCode::Char('j') => {
                if tab_index == 0 {
                    list_state.select(step_selection(list_state.selected(), profiles.len(), true));
                } else {
                    config_scroll = config_scroll.saturating_add(1);
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if tab_index == 0 {
                    list_state.select(step_selection(list_state.selected(), profiles.len(), false));
                } else {
                    config_scroll = config_scroll.saturating_sub(1);
                }
            }

            KeyCode::Char('s') if tab_index == 0 => {
                start_benchmark(state, config);
            }

            KeyCode::Char(' ') | KeyCode::Enter if tab_index == 0 => {
                let Some(selected) = list_state.selected().and_then(|i| profiles.get(i)) else {
                    continue;
                };
                let selected = selected.clone();

                config.active_profile.clone_from(&selected);
                if let Err(err) = config.save() {
                    shell_handoff::log_debug(&format!("dash could not save config: {err}"));
                }
                with_state(state, |s| s.active_profile_key.clone_from(&selected));

                // Space previews a profile in place; Enter also hands the
                // exports back to the shell and exits.
                if key.code == KeyCode::Enter {
                    return Ok(DashAction::Apply(selected));
                }
            }
            _ => {}
        }
    }
}

/// Spawns a background benchmark of every profile, if one is not already running.
fn start_benchmark(state: &Arc<Mutex<DashState>>, config: &AppConfig) {
    let already_running = with_state(state, |s| {
        if s.is_benchmarking_all {
            return true;
        }
        s.is_benchmarking_all = true;
        s.benchmark_results = None;
        false
    });
    if already_running {
        return;
    }

    let state = Arc::clone(state);
    let config = config.clone();
    tokio::spawn(async move {
        // The quiet variant: a spinner here would write to stderr while
        // ratatui owns the alternate screen.
        let results = crate::cmd::profile::benchmark_profiles_quiet(&config)
            .await
            .into_iter()
            .map(|r| (r.key, r.avg_ms.unwrap_or(u128::MAX)))
            .collect();

        with_state(&state, |s| {
            s.benchmark_results = Some(results);
            s.is_benchmarking_all = false;
        });
    });
}

#[allow(clippy::too_many_arguments)] // Draw state is inherently wide; grouping it would only rename the fields.
fn draw(
    frame: &mut ratatui::Frame,
    config: &AppConfig,
    i18n: &I18n,
    snapshot: &DashState,
    profiles: &[String],
    list_state: &mut ListState,
    tab_index: usize,
    config_scroll: u16,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .split(frame.area());

    let header_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(chunks[0]);

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            " ⚡ Terminal Session Proxy Manager ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("— {}", i18n.t("dash_title")),
            Style::default().fg(Color::White),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(header, header_chunks[0]);

    let tabs = Tabs::new(vec![
        Line::from(format!(" [1] {} ", i18n.t("dash_tab_profiles"))),
        Line::from(format!(" [2] {} ", i18n.t("dash_tab_config"))),
    ])
    .block(Block::default().borders(Borders::ALL))
    .select(tab_index)
    .highlight_style(
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    );
    frame.render_widget(tabs, header_chunks[1]);

    if tab_index == 0 {
        draw_profiles_tab(
            frame, config, i18n, snapshot, profiles, list_state, chunks[1],
        );
    } else {
        let config_json = serde_json::to_string_pretty(config).unwrap_or_default();
        let paragraph = Paragraph::new(config_json)
            .block(
                Block::default()
                    .title(format!(" {} ", i18n.t("dash_tab_config")))
                    .borders(Borders::ALL),
            )
            .scroll((config_scroll, 0));
        frame.render_widget(paragraph, chunks[1]);
    }

    let footer = Paragraph::new(Line::from(footer_spans(i18n, tab_index)))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(footer, chunks[2]);
}

fn draw_profiles_tab(
    frame: &mut ratatui::Frame,
    config: &AppConfig,
    i18n: &I18n,
    snapshot: &DashState,
    profiles: &[String],
    list_state: &mut ListState,
    area: ratatui::layout::Rect,
) {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(7), Constraint::Min(5)])
        .split(area);

    let middle_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(main_chunks[0]);

    let ping_span = match snapshot.ping_ms {
        Some(ms) if ms < 200 => styled(format!("{ms}ms"), Color::Green),
        Some(ms) if ms < 500 => styled(format!("{ms}ms"), Color::Yellow),
        Some(ms) => styled(format!("{ms}ms"), Color::Red),
        None => Span::styled(
            i18n.t("dash_offline").to_string(),
            Style::default().fg(Color::Red),
        ),
    };

    let value_style = if snapshot.is_loading {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    };

    let port_span = match snapshot.is_local_listening {
        Some(true) => styled(format!("{} 🟢", i18n.t("dash_port_active")), Color::Green),
        Some(false) => styled(format!("{} 🔴", i18n.t("dash_port_dead")), Color::Red),
        None => Span::styled(
            i18n.t("dash_checking").to_string(),
            Style::default().fg(Color::DarkGray),
        ),
    };

    let rows = [
        (
            i18n.t("dash_external_ip"),
            Span::styled(snapshot.ip.clone(), value_style),
        ),
        (
            i18n.t("dash_location"),
            Span::styled(snapshot.location.clone(), value_style),
        ),
        (i18n.t("dash_latency"), ping_span),
        (i18n.t("dash_local_port"), port_span),
    ];

    let mut info_text = vec![Line::from(Vec::new())];
    info_text.extend(rows.into_iter().map(|(label, value)| {
        Line::from(vec![
            Span::styled(format!(" {label:<14}"), Style::default().fg(Color::White)),
            value,
        ])
    }));

    frame.render_widget(
        Paragraph::new(info_text).block(
            Block::default()
                .title(format!(" {} ", i18n.t("dash_live_monitor")))
                .borders(Borders::ALL),
        ),
        middle_chunks[0],
    );

    frame.render_widget(
        Sparkline::default()
            .block(
                Block::default()
                    .title(format!(" {} ", i18n.t("dash_ping_history")))
                    .borders(Borders::ALL),
            )
            .data(&snapshot.ping_history)
            .style(Style::default().fg(Color::Green)),
        middle_chunks[1],
    );

    // `filter_map` rather than indexing: the profile list is re-sorted from a
    // background benchmark, and an indexing lookup here used to `unwrap()`.
    let items: Vec<ListItem> = profiles
        .iter()
        .filter_map(|key| {
            let profile = config.profiles.get(key)?;
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
            Some(ListItem::new(Line::from(Span::styled(content, style))))
        })
        .collect();

    let title = if snapshot.is_benchmarking_all {
        format!(
            " {} [{}] ",
            i18n.t("dash_profiles"),
            i18n.t("dash_benchmarking")
        )
    } else if profiles.is_empty() {
        format!(" {} — {} ", i18n.t("dash_profiles"), i18n.t("no_profiles"))
    } else {
        format!(" {} ", i18n.t("dash_profiles"))
    };

    frame.render_stateful_widget(
        List::new(items)
            .block(Block::default().title(title).borders(Borders::ALL))
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> "),
        main_chunks[1],
        list_state,
    );
}

fn styled(text: String, color: Color) -> Span<'static> {
    Span::styled(
        text,
        Style::default().fg(color).add_modifier(Modifier::BOLD),
    )
}

fn footer_spans(i18n: &I18n, tab_index: usize) -> Vec<Span<'static>> {
    let keys: &[(&str, &str, Color)] = if tab_index == 0 {
        &[
            ("↑↓", "dash_key_nav", Color::Cyan),
            ("Space", "dash_key_preview", Color::Cyan),
            ("Enter", "dash_key_apply", Color::Green),
            ("b", "dash_key_best", Color::Yellow),
            ("i", "dash_key_import", Color::Blue),
            ("e", "dash_key_edit", Color::Magenta),
            ("s", "dash_key_bench", Color::LightGreen),
            ("1/2", "dash_key_tabs", Color::Cyan),
            ("q", "dash_key_quit", Color::Red),
        ]
    } else {
        &[
            ("↑↓", "dash_key_scroll", Color::Cyan),
            ("1/2", "dash_key_tabs", Color::Cyan),
            ("e", "dash_key_edit", Color::Magenta),
            ("q", "dash_key_quit", Color::Red),
        ]
    };

    let mut spans = vec![Span::styled(" ", Style::default())];
    for (idx, (key, label, color)) in keys.iter().enumerate() {
        if idx > 0 {
            spans.push(Span::styled(" | ", Style::default().fg(Color::Gray)));
        }
        spans.push(Span::styled(
            (*key).to_string(),
            Style::default().fg(*color).add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(
            format!(" {}", i18n.t(label)),
            Style::default().fg(Color::Gray),
        ));
    }
    spans
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

    #[test]
    fn arrow_keys_do_nothing_on_an_empty_profile_list() {
        // This underflowed `len() - 1` and panicked inside the alternate
        // screen, which also left the terminal in raw mode.
        assert_eq!(step_selection(None, 0, true), None);
        assert_eq!(step_selection(Some(0), 0, false), None);
    }

    #[test]
    fn selection_advances_and_wraps_at_the_end() {
        assert_eq!(step_selection(Some(0), 3, true), Some(1));
        assert_eq!(step_selection(Some(2), 3, true), Some(0));
    }

    #[test]
    fn selection_wraps_backwards_past_the_start() {
        assert_eq!(step_selection(Some(0), 3, false), Some(2));
        assert_eq!(step_selection(Some(2), 3, false), Some(1));
    }

    #[test]
    fn a_stale_selection_past_the_end_is_clamped_rather_than_panicking() {
        // The list is re-sorted and can shrink while a selection is held.
        assert_eq!(step_selection(Some(99), 3, true), Some(0));
        assert_eq!(step_selection(Some(99), 3, false), Some(1));
    }

    #[test]
    fn a_single_profile_list_always_stays_on_its_only_entry() {
        assert_eq!(step_selection(Some(0), 1, true), Some(0));
        assert_eq!(step_selection(Some(0), 1, false), Some(0));
    }

    #[test]
    fn history_keeps_only_the_most_recent_window() {
        let mut history = Vec::new();
        for sample in 0..(PING_HISTORY_LEN as u64 + 10) {
            push_history(&mut history, sample);
        }

        assert_eq!(history.len(), PING_HISTORY_LEN);
        assert_eq!(*history.last().unwrap(), PING_HISTORY_LEN as u64 + 9);
        assert_eq!(history[0], 10);
    }

    #[test]
    fn state_access_survives_a_poisoned_lock() {
        let state = Arc::new(Mutex::new(DashState::default()));
        let poisoner = Arc::clone(&state);
        let _ = std::thread::spawn(move || {
            let _guard = poisoner.lock().unwrap();
            panic!("poison the mutex");
        })
        .join();

        assert!(state.is_poisoned());
        // Must not panic a second time.
        with_state(&state, |s| s.ip = "recovered".to_string());
        assert_eq!(with_state(&state, |s| s.ip.clone()), "recovered");
    }

    #[test]
    fn the_footer_offers_different_keys_per_tab() {
        let i18n = I18n::load("en");

        let profiles_tab = footer_spans(&i18n, 0).len();
        let config_tab = footer_spans(&i18n, 1).len();

        assert!(profiles_tab > config_tab);
    }
}
