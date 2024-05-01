use std::borrow::Cow;

use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use serde_json::Value;

use crate::identifier::Selector;

#[derive(Debug, Clone)]
pub struct JsonTreeItem<'json> {
    key: Selector,
    value: &'json Value,
    children: Vec<Self>,
}

impl<'json> JsonTreeItem<'json> {
    const fn raw_leaf(key: Selector, value: &'json Value) -> Self {
        Self {
            key,
            value,
            children: Vec::new(),
        }
    }

    const fn raw(identifier: Selector, value: &'json Value, children: Vec<Self>) -> Self {
        // no need for child identifier uniqueness check as JSON itself already prevents this
        Self {
            key: identifier,
            value,
            children,
        }
    }

    #[must_use]
    pub fn new(root: &'json Value) -> Vec<Self> {
        match root {
            Value::Array(array) => Self::from_array(array),
            Value::Object(object) => Self::from_object(object),
            _ => vec![Self::raw_leaf(Selector::None, root)],
        }
    }

    #[must_use]
    fn new_inner(key: Selector, value: &'json Value) -> Self {
        match value {
            Value::Object(object) if object.is_empty() => Self::raw_leaf(key, value),
            Value::Object(object) => Self::raw(key, value, Self::from_object(object)),
            Value::Array(array) if array.is_empty() => Self::raw_leaf(key, value),
            Value::Array(array) => Self::raw(key, value, Self::from_array(array)),
            _ => Self::raw_leaf(key, value),
        }
    }

    #[must_use]
    fn from_object(object: &'json serde_json::Map<String, Value>) -> Vec<Self> {
        object
            .iter()
            .map(|(key, value)| Self::new_inner(Selector::ObjectKey(key.clone()), value))
            .collect()
    }

    #[must_use]
    fn from_array(array: &'json [Value]) -> Vec<Self> {
        array
            .iter()
            .enumerate()
            .map(|(index, value)| Self::new_inner(Selector::ArrayIndex(index), value))
            .collect()
    }
}

impl crate::tree_item::TreeItem for JsonTreeItem<'_> {
    type Identifier = Selector;

    fn children(&self) -> &[Self] {
        &self.children
    }

    fn height(&self) -> usize {
        1
    }

    fn identifier(&self) -> &Self::Identifier {
        &self.key
    }

    fn render(&self, area: ratatui::layout::Rect, buffer: &mut ratatui::buffer::Buffer) {
        const KEY: Style = Style::new().fg(Color::Blue);
        const INDEX: Style = Style::new().fg(Color::Cyan);

        const NAME_SEPARATOR: Span = Span {
            content: Cow::Borrowed(": "),
            style: Style::new().fg(Color::DarkGray),
        };

        let value_span = get_value_span(self.value);
        let spans = match &self.key {
            Selector::ObjectKey(key) => vec![
                Span {
                    content: Cow::Borrowed(key),
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
        ratatui::widgets::Widget::render(Line::from(spans), area, buffer);
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

#[test]
fn empty_creates_empty_tree() {
    let json = serde_json::json!({});
    let tree_items = JsonTreeItem::new(&json);
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
        let items = JsonTreeItem::new(&json);
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
