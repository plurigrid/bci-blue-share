use std::collections::HashMap;
use std::io;
use std::time::Duration;

use bci_core::{Agg, App, Item};
use color_eyre::eyre::Result;
use crossterm::event::{Event, EventStream, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use futures::StreamExt;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

const DEFAULT_ENDPOINT: &str = "https://bci.blue";

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let endpoint = std::env::var("BCI_ENDPOINT").unwrap_or_else(|_| DEFAULT_ENDPOINT.into());
    let client = reqwest::Client::builder().timeout(Duration::from_secs(10)).build()?;
    let mut term = init_term()?;
    install_panic_hook();
    let mut app = App::new();
    refresh(&client, &endpoint, &mut app).await;
    let res = run_loop(&mut term, &mut app, &client, &endpoint).await;
    restore_term()?;
    res
}

async fn run_loop(
    term: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    client: &reqwest::Client,
    endpoint: &str,
) -> Result<()> {
    let mut events = EventStream::new();
    let mut tick = tokio::time::interval(Duration::from_secs(15));
    tick.tick().await;
    loop {
        term.draw(|f| bci_core::ui::draw(f, app))?;
        tokio::select! {
            maybe_event = events.next() => {
                if let Some(Ok(Event::Key(k))) = maybe_event {
                    if k.kind != KeyEventKind::Press { continue; }
                    match k.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Char('r') => vote(client, endpoint, app, -1).await,
                        KeyCode::Char('b') => vote(client, endpoint, app, 1).await,
                        KeyCode::Char(' ') => vote(client, endpoint, app, 0).await,
                        KeyCode::Down | KeyCode::Char('j') => app.next(),
                        KeyCode::Up | KeyCode::Char('k') => app.prev(),
                        KeyCode::Char('R') => refresh(client, endpoint, app).await,
                        _ => {}
                    }
                }
            }
            _ = tick.tick() => { refresh(client, endpoint, app).await; }
        }
    }
    Ok(())
}

async fn refresh(client: &reqwest::Client, endpoint: &str, app: &mut App) {
    match fetch_catalog(client, endpoint).await {
        Ok(items) => app.set_items(items),
        Err(e) => app.err(format!("catalog: {e}")),
    }
    match fetch_agg(client, endpoint).await {
        Ok(a) => app.set_aggs(a),
        Err(e) => app.err(format!("agg: {e}")),
    }
}

async fn vote(client: &reqwest::Client, endpoint: &str, app: &mut App, choice: i8) {
    let Some(item) = app.selected_item().cloned() else { return; };
    match cast_vote(client, endpoint, &item.id, choice).await {
        Ok(agg) => app.apply_vote(&item.id, agg, choice),
        Err(e) => app.err(format!("vote: {e}")),
    }
}

async fn fetch_catalog(client: &reqwest::Client, endpoint: &str) -> Result<Vec<Item>> {
    let url = format!("{}/api/catalog", endpoint.trim_end_matches('/'));
    let v: serde_json::Value = client.get(url).send().await?.json().await?;
    Ok(serde_json::from_value(v["items"].clone())?)
}
async fn fetch_agg(client: &reqwest::Client, endpoint: &str) -> Result<HashMap<String, Agg>> {
    let url = format!("{}/api/agg", endpoint.trim_end_matches('/'));
    let v: serde_json::Value = client.get(url).send().await?.json().await?;
    Ok(serde_json::from_value(v["aggregates"].clone()).unwrap_or_default())
}
async fn cast_vote(client: &reqwest::Client, endpoint: &str, item: &str, choice: i8) -> Result<Agg> {
    let url = format!("{}/api/vote", endpoint.trim_end_matches('/'));
    let body = serde_json::json!({"item": item, "choice": choice});
    let v: serde_json::Value = client.post(url).json(&body).send().await?.json().await?;
    Ok(serde_json::from_value(v["agg"].clone()).unwrap_or_default())
}

fn init_term() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    Terminal::new(CrosstermBackend::new(stdout))
}
fn restore_term() -> io::Result<()> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}
fn install_panic_hook() {
    let original = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        original(info);
    }));
}
