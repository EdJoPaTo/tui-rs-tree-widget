use std::borrow::Cow;

use ratatui::style::{Color, Style};
use ratatui::text::Span;

pub const KEY: Style = Style::new().fg(Color::Blue);
pub const INDEX: Style = Style::new().fg(Color::Cyan);

pub const NAME_SEPARATOR: Span = Span {
    content: Cow::Borrowed(": "),
    style: Style::new().fg(Color::DarkGray),
};

pub const BOOL: Style = Style::new().fg(Color::Magenta);
pub const NULL: Style = Style::new().fg(Color::DarkGray);
pub const NUMBER: Style = Style::new().fg(Color::LightBlue);
pub const STRING: Style = Style::new().fg(Color::Green);
