use std::{fmt::Display, io};

use crossterm::event::{self, Event, KeyCode};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};

use crate::docker;

#[derive(PartialEq)]
enum Location {
    Unknown,
    Catalog,
    Image,
}
impl Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Location::Unknown => "Unknown",
                Location::Catalog => "Catalog",
                Location::Image => "Image",
            }
        )
    }
}

enum MessageTitle {
    Tips,
    Search,
}
impl Display for MessageTitle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                MessageTitle::Tips => "Tips",
                MessageTitle::Search => "Search",
            }
        )
    }
}

enum Mode {
    Normal,
    Search,
}

pub struct App<'a> {
    focus_index: usize,
    message_title: MessageTitle,
    message: Vec<Span<'a>>,
    data: Vec<String>,
    current_location: Location,
    mode: Mode,
    input: String,
}

impl<'a> Default for App<'a> {
    fn default() -> Self {
        Self {
            focus_index: 0,
            message_title: MessageTitle::Tips,
            message: vec![],
            data: vec![],
            current_location: Location::Unknown,
            mode: Mode::Normal,
            input: String::new(),
        }
    }
}

pub fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    let catalog = docker::get_catalog();
    app.data = catalog.repositories;
    app.current_location = Location::Catalog;

    loop {
        terminal.draw(|f| ui(f, &app))?;

        if let Event::Key(key) = event::read()? {
            match app.mode {
                Mode::Normal => match key.code {
                    KeyCode::Char('q') => {
                        return Ok(());
                    }
                    KeyCode::Esc => match app.current_location {
                        Location::Catalog => {
                            return Ok(());
                        }
                        Location::Image => {
                            app.current_location = Location::Catalog;
                            app.data = docker::get_catalog().repositories;
                            app.focus_index = 0;
                        }
                        _ => {}
                    },
                    KeyCode::Down | KeyCode::Char('j') => {
                        if app.data.len() > app.focus_index {
                            app.focus_index += 1;
                        } else {
                            app.message = vec![Span::from("You reached bottom of the result")]
                        }
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        if app.focus_index > 0 {
                            app.focus_index -= 1;
                        } else {
                            app.message = vec![Span::from("You reached top of the result")]
                        }
                    }
                    KeyCode::Enter => match app.current_location {
                        Location::Catalog => {
                            app.current_location = Location::Image;
                            app.data = docker::get_image(
                                app.data[app.focus_index].clone()
                            ).tags;
                            app.focus_index = 0;
                        }
                        _ => {}
                    },
                    KeyCode::Char('/') => {
                        app.message_title = MessageTitle::Search;
                        app.mode = Mode::Search;
                    }
                    _ => {}
                },
                Mode::Search => match key.code {
                    KeyCode::Enter => {
                        app.message_title = MessageTitle::Tips;
                        app.mode = Mode::Normal;
                    }
                    KeyCode::Esc => {
                        app.message_title = MessageTitle::Tips;
                        app.mode = Mode::Normal;
                    }
                    KeyCode::Backspace => {
                        app.input = app.input[1..(app.input.len() - 1)].to_string()
                    }
                    _ => {}
                },
            }
        }
    }
}

pub fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([Constraint::Ratio(1, 5), Constraint::Ratio(4, 5)].as_ref())
        .split(f.size());

    let title = "Docker TUI";
    let title = Paragraph::new(Text::from(Spans::from(Span::styled(
        title,
        Style::default().add_modifier(Modifier::BOLD),
    ))));
    f.render_widget(
        title,
        Rect {
            x: 2,
            y: 0,
            width: 100,
            height: 10,
        },
    );

    let paragraph = Paragraph::new(Text::from(Spans::from(match app.message_title {
        MessageTitle::Tips => vec![
            Span::raw("Press "),
            Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" to exit."),
        ],
        _ => app.message.clone(),
    })));
    f.render_widget(
        paragraph.block(
            Block::default()
                .borders(Borders::ALL)
                .title(app.message_title.to_string()),
        ),
        chunks[0],
    );
    f.render_widget(
        List::new(
            app.data
                .clone()
                .iter()
                .enumerate()
                .map(|(idx, it)| {
                    let style = if idx == app.focus_index {
                        Style::default().bg(Color::White).fg(Color::Black)
                    } else {
                        Style::default()
                    };
                    ListItem::new(Spans::from(Span::styled(format!("{}", it), style)))
                })
                .collect::<Vec<ListItem>>(),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(app.current_location.to_string()),
        ),
        chunks[1],
    )
}
