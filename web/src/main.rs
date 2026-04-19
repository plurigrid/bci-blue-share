use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use bci_core::{Agg, App, Item};
use ratzilla::ratatui::Terminal;
use ratzilla::{event::KeyCode, DomBackend, WebRenderer};

fn main() -> std::io::Result<()> {
    console_error_panic_hook::set_once();
    let backend = DomBackend::new()?;
    let term = Terminal::new(backend)?;
    let app = Rc::new(RefCell::new(App::new()));

    spawn_fetch_all(app.clone());

    {
        let app = app.clone();
        wasm_bindgen_futures::spawn_local(async move {
            loop {
                gloo_timers::future::TimeoutFuture::new(15_000).await;
                spawn_fetch_all(app.clone());
            }
        });
    }

    term.on_key_event({
        let app = app.clone();
        move |k| match k.code {
            KeyCode::Char('r') => vote(app.clone(), -1),
            KeyCode::Char('b') => vote(app.clone(), 1),
            KeyCode::Char(' ') => vote(app.clone(), 0),
            KeyCode::Char('R') => spawn_fetch_all(app.clone()),
            KeyCode::Down | KeyCode::Char('j') => app.borrow_mut().next(),
            KeyCode::Up | KeyCode::Char('k') => app.borrow_mut().prev(),
            _ => {}
        }
    });

    term.draw_web(move |f| {
        let a = app.borrow();
        bci_core::ui::draw(f, &a);
    });

    Ok(())
}

fn spawn_fetch_all(app: Rc<RefCell<App>>) {
    wasm_bindgen_futures::spawn_local(async move {
        match fetch_catalog().await {
            Ok(items) => app.borrow_mut().set_items(items),
            Err(e) => app.borrow_mut().err(format!("catalog: {e}")),
        }
        match fetch_agg().await {
            Ok(a) => app.borrow_mut().set_aggs(a),
            Err(e) => app.borrow_mut().err(format!("agg: {e}")),
        }
    });
}

fn vote(app: Rc<RefCell<App>>, choice: i8) {
    let item = app.borrow().selected_item().cloned();
    if let Some(item) = item {
        wasm_bindgen_futures::spawn_local(async move {
            match cast_vote(&item.id, choice).await {
                Ok(agg) => app.borrow_mut().apply_vote(&item.id, agg, choice),
                Err(e) => app.borrow_mut().err(format!("vote: {e}")),
            }
        });
    }
}

async fn fetch_catalog() -> Result<Vec<Item>, String> {
    let r = gloo_net::http::Request::get("/api/catalog").send().await.map_err(|e| e.to_string())?;
    let v: serde_json::Value = r.json().await.map_err(|e| e.to_string())?;
    serde_json::from_value(v["items"].clone()).map_err(|e| e.to_string())
}
async fn fetch_agg() -> Result<HashMap<String, Agg>, String> {
    let r = gloo_net::http::Request::get("/api/agg").send().await.map_err(|e| e.to_string())?;
    let v: serde_json::Value = r.json().await.map_err(|e| e.to_string())?;
    Ok(serde_json::from_value(v["aggregates"].clone()).unwrap_or_default())
}
async fn cast_vote(item: &str, choice: i8) -> Result<Agg, String> {
    let body = serde_json::json!({"item": item, "choice": choice});
    let r = gloo_net::http::Request::post("/api/vote")
        .header("Content-Type", "application/json")
        .body(body.to_string()).map_err(|e| e.to_string())?
        .send().await.map_err(|e| e.to_string())?;
    let v: serde_json::Value = r.json().await.map_err(|e| e.to_string())?;
    Ok(serde_json::from_value(v["agg"].clone()).unwrap_or_default())
}
