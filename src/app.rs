use std::{
    cmp::{max, min},
    fmt::Display,
    io,
};

use crossterm::event::{self, Event, KeyCode};
use regex::Regex;
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
    Image(String),
}
impl Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Location::Unknown => "Unknown".to_string(),
                Location::Catalog => "Catalog".to_string(),
                Location::Image(name) => name.to_string(),
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
    focus_index: i32,
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

pub fn get_random_elem<T: Clone>(vec: &Vec<T>) -> T {
    vec[(rand::random::<f32>() * vec.len() as f32).floor() as usize].clone()
}

pub fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    let help_messages = vec![vec![
        Span::raw("Press "),
        Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" to exit."),
    ]];

    let catalog = docker::get_catalog();
    app.data = catalog.repositories;
    app.current_location = Location::Catalog;
    app.message = get_random_elem(&help_messages);

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
                            app.data = docker::get_catalog().repositories;
                            app.focus_index = 0;
                        }
                        Location::Image(_) => {
                            app.current_location = Location::Catalog;
                            app.data = docker::get_catalog().repositories;
                            app.focus_index = 0;
                        }
                        _ => {}
                    },
                    KeyCode::Down | KeyCode::Char('j') => {
                        if app.data.len() - 1 > app.focus_index as usize {
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
                    KeyCode::Char('G') => {
                        app.focus_index = app.data.len() as i32 - 1;
                    }
                    KeyCode::Enter => match app.current_location {
                        Location::Catalog => {
                            let target_image_name = app.data[app.focus_index as usize].clone();
                            app.current_location = Location::Image(target_image_name.clone());
                            app.data = docker::get_image(target_image_name).tags;
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
                        app.focus_index = 0;
                        app.data = app
                            .data
                            .iter()
                            .filter(|it| {
                                let pattern = Regex::new(&app.input).unwrap();
                                pattern.is_match(it)
                            })
                            .map(|it| it.to_string())
                            .collect();
                        app.message = get_random_elem(&help_messages);
                    }
                    KeyCode::Esc => {
                        app.message_title = MessageTitle::Tips;
                        app.mode = Mode::Normal;
                        match app.current_location {
                            Location::Catalog => app.data = docker::get_catalog().repositories,
                            Location::Image(ref image_name) => {
                                app.data = docker::get_image(image_name.to_string()).tags;
                            }
                            Location::Unknown => {}
                        }
                        app.input = String::new();
                        app.message = get_random_elem(&help_messages);
                    }
                    KeyCode::Backspace => {
                        if app.input.len() > 0 {
                            app.input = app.input[0..(app.input.len() - 1)].to_string();
                            app.message = vec![Span::raw(app.input.clone())];
                        }
                    }
                    code => {
                        if let KeyCode::Char(character) = code {
                            app.input = app.input + &character.to_string();
                            app.message = vec![Span::raw(app.input.clone())];
                        }
                    }
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

    let message = match app.message_title {
        MessageTitle::Tips => Paragraph::new(Text::from(Spans::from(app.message.clone()))),
        MessageTitle::Search => {
            Paragraph::new(Text::from(Spans::from(vec![Span::raw(app.input.clone())])))
        }
    };
    f.render_widget(
        message.block(
            Block::default()
                .borders(Borders::ALL)
                .title(app.message_title.to_string()),
        ),
        chunks[0],
    );

    let start_idx = max(min(app.focus_index, app.data.len() as i32 - 20), 0 );
    let end_idx = min(start_idx + 21, app.data.len() as i32 - 1);

    f.render_widget(
        List::new(
            app.data
                .clone()
                .iter()
                .enumerate()
                .map(|(idx, it)| {
                    let style = if idx == app.focus_index as usize {
                        Style::default().bg(Color::White).fg(Color::Black)
                    } else {
                        Style::default()
                    };
                    ListItem::new(Spans::from(Span::styled(format!("{}", it), style)))
                })
                .enumerate()
                .fold(vec![], |acc, (idx, it)| {
                    if idx >= start_idx as usize && idx <= end_idx as usize {
                        [acc, vec![it]].concat()
                    } else {
                        acc
                    }
                }),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(app.current_location.to_string()),
        ),
        chunks[1],
    )
}
