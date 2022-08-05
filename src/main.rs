mod app;
mod docker;

use std::{io, fs::File, env, str::FromStr};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{backend::CrosstermBackend, Terminal};

fn main() -> Result<(), io::Error> {
    let now = chrono::Local::now();
    let log_level = env::var("LOG_LEVEL").unwrap_or("INFO".to_string());
    let log_buffer = File::create(format!("log/{}.log", now.to_rfc3339()))?;

    let _ = simplelog::WriteLogger::init(
        log::LevelFilter::from_str(&log_level).unwrap(),
        simplelog::Config::default(), 
        log_buffer,
    );

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app = app::App::default();
    let res = app::run_app(&mut terminal, app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
    )?;
    terminal.show_cursor()?;

    if let Err(e) = res {
        println!("{:?}", e);
    }

    Ok(())
}
