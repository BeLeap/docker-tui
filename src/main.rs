mod app;
mod docker;

use std::io;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use log::LevelFilter;
use log4rs::{append::file::FileAppender, encode::pattern::PatternEncoder, Config, config::{Appender, Root}};
use tui::{backend::CrosstermBackend, Terminal};

fn main() -> Result<(), io::Error> {
    let file = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} - {m}{n}")))
        .build("log/requests.log")
        .unwrap();
    let config = Config::builder()
        .appender(Appender::builder().build("file", Box::new(file)))
        .build(Root::builder()
            .appender("file")
            .build(LevelFilter::Debug))
        .unwrap();
    let _handle = log4rs::init_config(config).unwrap();

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
