// ══════════════════════════════════════════════════════════════════════════
//  Ng'anjo OS — Power TUI v2.0  (Rust / ratatui rewrite)
//  Entry point: wires up the terminal, spawns the background data thread,
//  and runs the main event loop.
// ══════════════════════════════════════════════════════════════════════════

mod app;
mod ui;
mod data;
mod panels;
mod modal;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{
    io,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use app::{App, Tab};
use data::SystemData;

fn main() -> Result<()> {
    // ── terminal setup ────────────────────────────────────────────────────
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend  = CrosstermBackend::new(stdout);
    let mut term = Terminal::new(backend)?;

    // ── shared state ──────────────────────────────────────────────────────
    let sysdata = Arc::new(Mutex::new(SystemData::new()));
    let mut app = App::new(Arc::clone(&sysdata));

    // ── background refresh thread ─────────────────────────────────────────
    // Updates CPU, RAM, disk, network, process list every 2 s without
    // blocking the UI thread.
    {
        let sysdata = Arc::clone(&sysdata);
        thread::spawn(move || {
            loop {
                {
                    if let Ok(mut d) = sysdata.lock() {
                        d.refresh();
                    }
                }
                thread::sleep(Duration::from_secs(2));
            }
        });
    }

    // ── main event loop ───────────────────────────────────────────────────
    let tick = Duration::from_millis(250);
    loop {
        term.draw(|f| ui::draw(f, &mut app))?;

        if event::poll(tick)? {
            if let Event::Key(key) = event::read()? {
                // global quit
                if key.code == KeyCode::Char('q')
                    && key.modifiers == KeyModifiers::NONE
                    && app.modal.is_none()
                {
                    break;
                }
                app.handle_key(key);
            }
        }

        if app.should_quit {
            break;
        }
    }

    // ── cleanup ───────────────────────────────────────────────────────────
    disable_raw_mode()?;
    execute!(
        term.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    term.show_cursor()?;
    Ok(())
}
