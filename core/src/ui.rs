use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;

use crate::types::Item;
use crate::App;

pub fn draw(f: &mut Frame, app: &App) {
    let [top, mid, scope, status, help] = Layout::vertical([
        Constraint::Length(1), Constraint::Min(8), Constraint::Length(5),
        Constraint::Length(1), Constraint::Length(1),
    ]).areas(f.area());

    f.render_widget(
        Paragraph::new(Line::from(vec![
            "bci.red ".bold().red(), "⊕ ".dim(), "bci.blue ".bold().blue(),
            "· oscilloscope ".dim(), format!("({} items)", app.items.len()).dim(),
        ])),
        top,
    );

    let red: Vec<&Item> = app.items.iter().filter(|i| i.bin == "red").collect();
    let blue: Vec<&Item> = app.items.iter().filter(|i| i.bin == "blue").collect();
    let [left, right] = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(mid);
    render_bin(f, left, "bci.red", &red, app, true);
    render_bin(f, right, "bci.blue", &blue, app, false);

    render_scope(f, scope, app);

    let t = app.totals();
    f.render_widget(
        Paragraph::new(Line::from(vec![
            " red ".bold().red(), format!("{} ", t.red).into(),
            "blue ".bold().blue(), format!("{} ", t.blue).into(),
            "abstain ".bold().dim(), format!("{} ", t.abstain).into(),
            "total ".bold(), format!("{} ", t.total).into(),
            "gf3 ".bold(), format!("{} ", app.gf3()).cyan(),
            "  ".into(), app.last_msg.clone().dim(),
        ])),
        status,
    );

    f.render_widget(
        Paragraph::new(" [r] vote red   [b] vote blue   [⎵] abstain   [↑↓/jk] navigate   [R] refresh   [q] quit ".dim()),
        help,
    );
}

fn render_bin(f: &mut Frame, area: Rect, title: &str, items: &[&Item], app: &App, is_red: bool) {
    let lines: Vec<ListItem> = items.iter().map(|it| {
        let agg = app.aggs.get(&it.id).cloned().unwrap_or_default();
        let net = agg.blue as i64 - agg.red as i64;
        let gi = app.items.iter().position(|x| x.id == it.id).unwrap_or(0);
        let cur = if gi == app.selected { "▶" } else { " " };
        let text = format!("{} {}  {}  ↑{} ↓{} ø{} net {:+}", cur, it.id, it.title, agg.blue, agg.red, agg.abstain, net);
        let mut item = ListItem::new(text);
        if gi == app.selected { item = item.style(Style::new().bold().on_dark_gray()); }
        item
    }).collect();
    let total: u64 = items.iter().map(|it| app.aggs.get(&it.id).map(|a| a.total).unwrap_or(0)).sum();
    let title_full = format!(" {} ── {} votes ", title, total);
    let block = Block::default().borders(Borders::ALL).title(title_full.bold());
    let block = if is_red { block.border_style(Style::new().red()) } else { block.border_style(Style::new().blue()) };
    f.render_widget(List::new(lines).block(block), area);
}

fn render_scope(f: &mut Frame, area: Rect, app: &App) {
    let bars = ['▁','▂','▃','▄','▅','▆','▇','█'];
    let max = app.history.iter().map(|x| x.unsigned_abs()).max().unwrap_or(1).max(1);
    let line: String = app.history.iter().map(|&v| {
        let r = (v.unsigned_abs() as f64) / (max as f64);
        bars[((r * 7.0) as usize).min(7)]
    }).collect();
    f.render_widget(
        Paragraph::new(line.cyan()).block(Block::default().borders(Borders::ALL).title(" scope: net (blue − red), 60s window ".bold())),
        area,
    );
}
