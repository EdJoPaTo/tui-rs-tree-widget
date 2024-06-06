/*! Common building blocks like colors for common elements like data types */

use std::borrow::Cow;

use ratatui::style::{Color, Style};
use ratatui::text::Span;

/// Generic key of key value data structures
pub const KEY: Style = Style::new().fg(Color::Blue);
/// Index in arrays
pub const INDEX: Style = Style::new().fg(Color::Cyan);

/// Separator between key and value
pub const NAME_SEPARATOR: Span = Span {
    content: Cow::Borrowed(": "),
    style: Style::new().fg(Color::DarkGray),
};

pub const BOOL: Style = Style::new().fg(Color::Magenta);
pub const NULL: Style = Style::new().fg(Color::DarkGray);
pub const NUMBER: Style = Style::new().fg(Color::LightBlue);
pub const STRING: Style = Style::new().fg(Color::Green);
