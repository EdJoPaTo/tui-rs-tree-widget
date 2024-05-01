use std::borrow::Cow;

use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use serde_json::Value;

use crate::identifier::Selector;
use crate::TreeItem;

/// Create [`TreeItem`]s from a [JSON](Value).
#[must_use]
pub fn tree_items(root: &Value) -> Vec<TreeItem<'_, Selector>> {
    match root {
        Value::Object(object) => from_object(object),
        Value::Array(array) => from_array(array),
        _ => vec![TreeItem::new_leaf(Selector::None, get_value_span(root))],
    }
}

fn get_value_span(value: &Value) -> Span {
    const BOOL: Style = Style::new().fg(Color::Magenta);
    const NULL: Style = Style::new().fg(Color::DarkGray);
    const NUMBER: Style = Style::new().fg(Color::LightBlue);
    const STRING: Style = Style::new().fg(Color::Green);

    match value {
        Value::Array(array) if array.is_empty() => Span {
            content: Cow::Borrowed("[]"),
            style: Style::new(),
        },
        Value::Array(_) => Span {
            content: Cow::Borrowed("["),
            style: Style::new(),
        },
        Value::Object(object) if object.is_empty() => Span {
            content: Cow::Borrowed("{}"),
            style: Style::new(),
        },
        Value::Object(_) => Span {
            content: Cow::Borrowed("{"),
            style: Style::new(),
        },
        Value::Null => Span {
            content: Cow::Borrowed("null"),
            style: NULL,
        },
        Value::Bool(true) => Span {
            content: Cow::Borrowed("true"),
            style: BOOL,
        },
        Value::Bool(false) => Span {
            content: Cow::Borrowed("false"),
            style: BOOL,
        },
        Value::Number(number) => Span {
            content: Cow::Owned(number.to_string()),
            style: NUMBER,
        },
        Value::String(string) => Span {
            content: Cow::Borrowed(string),
            style: STRING,
        },
    }
}

fn recurse(key: Selector, value: &Value) -> TreeItem<Selector> {
    const KEY: Style = Style::new().fg(Color::Blue);
    const INDEX: Style = Style::new().fg(Color::Cyan);

    const NAME_SEPARATOR: Span = Span {
        content: Cow::Borrowed(": "),
        style: Style::new().fg(Color::DarkGray),
    };

    let value_span = get_value_span(value);
    let spans = match key {
        Selector::ObjectKey(ref key) => vec![
            Span {
                content: Cow::Owned(key.clone()),
                style: KEY,
            },
            NAME_SEPARATOR,
            value_span,
        ],
        Selector::ArrayIndex(index) => vec![
            Span {
                content: Cow::Owned(index.to_string()),
                style: INDEX,
            },
            NAME_SEPARATOR,
            value_span,
        ],
        Selector::None => vec![value_span],
    };
    let text = Line::from(spans);

    match value {
        Value::Array(array) if !array.is_empty() => {
            TreeItem::new(key, text, from_array(array)).unwrap()
        }
        Value::Object(object) if !object.is_empty() => {
            TreeItem::new(key, text, from_object(object)).unwrap()
        }
        _ => TreeItem::new_leaf(key, text),
    }
}

fn from_object(object: &serde_json::Map<String, Value>) -> Vec<TreeItem<'_, Selector>> {
    object
        .iter()
        .map(|(key, value)| recurse(Selector::ObjectKey(key.clone()), value))
        .collect()
}

fn from_array(array: &[Value]) -> Vec<TreeItem<'_, Selector>> {
    array
        .iter()
        .enumerate()
        .map(|(index, value)| recurse(Selector::ArrayIndex(index), value))
        .collect()
}

#[test]
fn empty_creates_empty_tree() {
    let json = serde_json::json!({});
    let tree_items = tree_items(&json);
    dbg!(&tree_items);
    assert!(tree_items.is_empty());
}

#[cfg(test)]
mod render_tests {
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;

    use super::*;
    use crate::{Tree, TreeState};

    fn key(key: &str) -> Selector {
        Selector::ObjectKey(key.to_owned())
    }

    /// Strips colors after render
    #[must_use]
    #[track_caller]
    fn render(width: u16, height: u16, json: &str, state: &mut TreeState<Selector>) -> Buffer {
        let json = serde_json::from_str(json).expect("invalid test JSON");
        let items = tree_items(&json);
        let tree = Tree::new(items).unwrap().highlight_symbol(">> ");
        let area = Rect::new(0, 0, width, height);
        let mut buffer = Buffer::empty(area);
        ratatui::widgets::StatefulWidget::render(tree, area, &mut buffer, state);
        buffer.set_style(area, Style::reset());
        buffer
    }

    #[test]
    fn empty_array_renders_nothing() {
        let buffer = render(5, 3, "[]", &mut TreeState::default());
        let expected = Buffer::with_lines(["     "; 3]);
        assert_eq!(buffer, expected);
    }

    #[test]
    fn empty_object_renders_nothing() {
        let buffer = render(5, 3, "{}", &mut TreeState::default());
        let expected = Buffer::with_lines(["     "; 3]);
        assert_eq!(buffer, expected);
    }

    #[test]
    fn number() {
        let buffer = render(5, 2, "42", &mut TreeState::default());
        let expected = Buffer::with_lines(["  42 ", ""]);
        assert_eq!(buffer, expected);
    }

    #[test]
    fn all_simple_in_array() {
        let json = r#"[null, true, false, [], {}, 42, "lalala"]"#;
        let buffer = render(12, 8, json, &mut TreeState::default());
        let expected = Buffer::with_lines([
            "  0: null   ",
            "  1: true   ",
            "  2: false  ",
            "  3: []     ",
            "  4: {}     ",
            "  5: 42     ",
            "  6: lalala ",
            "            ",
        ]);
        assert_eq!(buffer, expected);
    }

    #[test]
    fn bigger_example() {
        let mut state = TreeState::default();
        state.open(vec![key("foo")]);
        state.open(vec![key("foo"), key("bar")]);

        let json = r#"{"foo": {"bar": [13, 37]}, "test": true}"#;
        let buffer = render(14, 6, json, &mut state);
        let expected = Buffer::with_lines([
            "▼ foo: {      ",
            "  ▼ bar: [    ",
            "      0: 13   ",
            "      1: 37   ",
            "  test: true  ",
            "              ",
        ]);
        assert_eq!(buffer, expected);
    }

    #[test]
    fn bigger_example_selection() {
        let mut state = TreeState::default();
        state.open(vec![key("foo")]);
        state.open(vec![key("foo"), key("bar")]);
        state.select(vec![key("foo"), key("bar"), Selector::ArrayIndex(1)]);

        let json = r#"{"foo": {"bar": [13, 37]}, "test": true}"#;
        let buffer = render(17, 6, json, &mut state);
        let expected = Buffer::with_lines([
            "   ▼ foo: {      ",
            "     ▼ bar: [    ",
            "         0: 13   ",
            ">>       1: 37   ",
            "     test: true  ",
            "                 ",
        ]);
        assert_eq!(buffer, expected);
    }
}
