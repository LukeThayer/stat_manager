//! stat_tui - Interactive TUI for stat management testing and visualization

mod app;
mod ui;
mod simulation;
mod commands;

use app::App;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

fn main() -> io::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new();

    // Main loop
    loop {
        terminal.draw(|f| ui::draw(f, &app))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match (key.code, key.modifiers) {
                    (KeyCode::Char('q'), _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => break,
                    (KeyCode::Tab, _) => app.next_tab(),
                    (KeyCode::BackTab, _) => app.prev_tab(),
                    (KeyCode::Char('1'), _) => app.set_tab(0),
                    (KeyCode::Char('2'), _) => app.set_tab(1),
                    (KeyCode::Char('3'), _) => app.set_tab(2),
                    (KeyCode::Char('4'), _) => app.set_tab(3),
                    (KeyCode::Char('5'), _) => app.set_tab(4),
                    (KeyCode::Char('6'), _) => app.set_tab(5),
                    (KeyCode::Up, _) | (KeyCode::Char('k'), _) => app.on_up(),
                    (KeyCode::Down, _) | (KeyCode::Char('j'), _) => app.on_down(),
                    (KeyCode::Left, _) | (KeyCode::Char('h'), _) => app.on_left(),
                    (KeyCode::Right, _) | (KeyCode::Char('l'), _) => app.on_right(),
                    (KeyCode::Enter, _) => app.on_enter(),
                    (KeyCode::Char(' '), _) => app.on_space(),
                    (KeyCode::Char('a'), _) => app.attack(),
                    (KeyCode::Char('t'), _) => app.tick_time(1.0),
                    (KeyCode::Char('r'), _) => app.reset(),
                    (KeyCode::Char('e'), _) => app.toggle_equip_target(),
                    (KeyCode::Char('u'), _) => app.unequip_current_slot(),
                    (KeyCode::Char('?'), _) => app.toggle_help(),
                    _ => {}
                }
            }
        }

        // Tick simulation
        app.tick(0.1);
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
