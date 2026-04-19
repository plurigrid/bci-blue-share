mod api;
mod app;
mod trit;
mod ui;

use std::io;
use std::time::{Duration, Instant};

use color_eyre::eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use app::App;

fn main() -> Result<()> {
    color_eyre::install()?;
    let endpoint = std::env::var("OSC_ENDPOINT").unwrap_or_else(|_| "https://bci.blue".to_string());
    let mut terminal = init_term()?;
    install_panic_hook();
    let mut app = App::new(endpoint);
    let _ = app.refresh();
    let res = run_loop(&mut terminal, &mut app);
    restore_term()?;
    res
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

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> Result<()> {
    let mut last_refresh = Instant::now();
    loop {
        terminal.draw(|f| ui::draw(f, app))?;
        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(k) = event::read()? {
                if k.kind != KeyEventKind::Press {
                    continue;
                }
                match k.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char('r') => { let _ = app.vote(-1); }
                    KeyCode::Char('b') => { let _ = app.vote(1); }
                    KeyCode::Char(' ') => { let _ = app.vote(0); }
                    KeyCode::Down | KeyCode::Char('j') => app.next(),
                    KeyCode::Up | KeyCode::Char('k') => app.prev(),
                    KeyCode::Char('R') => { let _ = app.refresh(); }
                    _ => {}
                }
            }
        }
        if last_refresh.elapsed() > Duration::from_secs(15) {
            let _ = app.refresh();
            last_refresh = Instant::now();
        }
    }
    Ok(())
}
