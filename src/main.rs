use clap::Parser;
use ratatui::{
    DefaultTerminal,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
};
use std::io;
use vedtoob::{app::App, pandoc_available, ui};

#[derive(Parser, Debug)]
#[command(
    name = "vedtoob",
    version,
    about = "TUI browser for Boot.dev courses",
    long_about = "TUI browser for Boot.dev courses\n\nControls:\n  q: quit\n  Esc: return to courses list\n  /: search courses\n  Enter: select\n  h/l: back/forward\n  j/k: down/up\n\nDependencies:\n  pandoc (required on PATH)\n  network access to api.boot.dev"
)]
struct Cli {}

fn main() -> io::Result<()> {
    let _cli = Cli::parse();

    if !pandoc_available() {
        eprintln!("Error: pandoc is required but not found in PATH");
        eprintln!("See https://github.com/jgm/pandoc");
        std::process::exit(1);
    }

    let mut terminal = ratatui::init();
    let result = run(&mut terminal);
    ratatui::restore();
    result
}

fn run(terminal: &mut DefaultTerminal) -> io::Result<()> {
    let mut app = App::new();

    loop {
        terminal.draw(|frame| ui::render(&mut app, frame))?;

        if let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            if app.is_search_mode {
                match key.code {
                    KeyCode::Esc => app.exit_search(),
                    KeyCode::Enter => app.submit_search(),
                    KeyCode::Backspace => app.pop_search(),
                    KeyCode::Char(c) => app.append_search(c),
                    _ => {}
                }
            } else {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Esc => app.back_to_courses(),
                    KeyCode::Left | KeyCode::Char('h') => app.go_back(),
                    KeyCode::Up | KeyCode::Char('k') => app.move_up(),
                    KeyCode::Down | KeyCode::Char('j') => app.move_down(),
                    KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => app.select(),
                    KeyCode::Char('/') => app.enter_search(),
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
