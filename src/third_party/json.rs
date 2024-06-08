use std::borrow::Cow;

use ratatui::style::Style;
use ratatui::text::{Line, Span};
use serde_json::Value;

use crate::key_value_tree_item::KeyValueTreeItem;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Selector {
    Key(String),
    Index(usize),
}

#[cfg(test)]
fn key(key: &str) -> Selector {
    Selector::Key(key.to_owned())
}

#[cfg(test)]
const fn index(index: usize) -> Selector {
    Selector::Index(index)
}

impl Selector {
    #[cfg(feature = "jsonptr")]
    #[must_use]
    pub fn as_jsonptr_token(&self) -> jsonptr::Token {
        match self {
            Self::Key(key) => jsonptr::Token::new(key),
            Self::Index(index) => jsonptr::Token::new(index.to_string()),
        }
    }

    #[cfg(feature = "jsonptr")]
    #[must_use]
    pub fn as_jsonptr(selector: &[Self]) -> jsonptr::Pointer {
        let token = selector
            .iter()
            .map(Self::as_jsonptr_token)
            .collect::<Vec<_>>();
        jsonptr::Pointer::new(token)
    }
}

fn get_value_span(value: &Value) -> Span {
    use super::common::{BOOL, NULL, NUMBER, STRING};

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

impl KeyValueTreeItem for Value {
    type Key = Selector;

    fn keys_below(&self) -> Vec<Self::Key> {
        match self {
            Self::Null | Self::Bool(_) | Self::Number(_) | Self::String(_) => Vec::new(),
            Self::Array(array) => array
                .iter()
                .enumerate()
                .map(|(index, _)| Selector::Index(index))
                .collect(),
            Self::Object(object) => object
                .keys()
                .map(|key| Selector::Key(key.to_owned()))
                .collect(),
        }
    }

    fn has_children(&self) -> bool {
        match self {
            Self::Array(array) if !array.is_empty() => true,
            Self::Object(object) if !object.is_empty() => true,
            _ => false,
        }
    }

    fn get_value(&self, key: &Self::Key) -> Option<&Self> {
        match (key, self) {
            (Selector::Key(key), Self::Object(object)) => object.get(key),
            (Selector::Index(index), Self::Array(array)) => array.get(*index),
            _ => None,
        }
    }

    fn get_children(&self) -> Vec<(Self::Key, &Self)> {
        match self {
            Self::Null | Self::Bool(_) | Self::Number(_) | Self::String(_) => Vec::new(),
            Self::Array(array) => array
                .iter()
                .enumerate()
                .map(|(index, value)| (Selector::Index(index), value))
                .collect(),
            Self::Object(object) => object
                .iter()
                .map(|(key, value)| (Selector::Key(key.to_owned()), value))
                .collect(),
        }
    }

    fn height(&self) -> usize {
        1
    }

    fn render(
        &self,
        key: Option<&Self::Key>,
        area: ratatui::layout::Rect,
        buffer: &mut ratatui::buffer::Buffer,
    ) {
        use super::common::{INDEX, KEY, NAME_SEPARATOR};

        let value_span = get_value_span(self);
        let spans = if let Some(key) = key {
            let key = match key {
                Selector::Key(key) => Span {
                    content: Cow::Borrowed(key),
                    style: KEY,
                },
                Selector::Index(index) => Span {
                    content: Cow::Owned(index.to_string()),
                    style: INDEX,
                },
            };
            vec![key, NAME_SEPARATOR, value_span]
        } else {
            vec![value_span]
        };
        let text = Line::from(spans);
        ratatui::widgets::Widget::render(&text, area, buffer);
    }
}

#[cfg(test)]
mod tree_data_tests {
    use std::collections::HashSet;

    use super::*;
    use crate::{Node, TreeData};

    #[track_caller]
    fn node<V>(identifier: V, has_children: bool) -> Node<Vec<Selector>>
    where
        V: AsRef<[Selector]>,
    {
        let identifier = identifier.as_ref();
        Node {
            depth: identifier.len().saturating_sub(1),
            has_children,
            height: 1,
            identifier: identifier.to_vec(),
        }
    }

    #[test]
    fn node_helper_works() {
        let result = node([key("foo"), key("bar")], false);
        let expected = Node {
            depth: 1,
            has_children: false,
            height: 1,
            identifier: vec![key("foo"), key("bar")],
        };
        assert_eq!(result, expected);
    }

    #[track_caller]
    fn case(json: &str) -> Vec<Node<Vec<Selector>>> {
        let mut open = HashSet::new();
        open.insert(vec![key("foo")]);
        open.insert(vec![key("foo"), key("bar")]);

        let json: Value = serde_json::from_str(json).expect("invalid JSON string");
        json.get_nodes(&open)
    }

    #[test]
    fn empty_array_has_empty_node() {
        assert_eq!(case("[]"), [node([], false)]);
    }

    #[test]
    fn empty_object_has_empty_node() {
        assert_eq!(case("{}"), [node([], false)]);
    }

    #[test]
    fn number_has_single_node() {
        assert_eq!(case("42"), [node([], false)]);
    }

    #[test]
    fn root_array_has_multiple_nodes() {
        assert_eq!(
            case("[13, 37]"),
            [node([index(0)], false), node([index(1)], false),]
        );
    }

    #[test]
    fn root_object_has_multiple_nodes() {
        assert_eq!(
            case(r#"{"foo": "bar", "something": true}"#),
            [node([key("foo")], false), node([key("something")], false),]
        );
    }

    #[test]
    fn deep_example() {
        assert_eq!(
            case(r#"{"foo": {"bar": [13, 37]}, "something": [42]}"#),
            [
                node([key("foo")], true),             // open
                node([key("foo"), key("bar")], true), // open
                node([key("foo"), key("bar"), index(0)], false),
                node([key("foo"), key("bar"), index(1)], false),
                node([key("something")], true),
            ]
        );
    }
}

#[cfg(test)]
mod render_tests {
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;

    use super::*;
    use crate::{Tree, TreeState};

    /// Strips colors after render
    #[must_use]
    #[track_caller]
    fn render(width: u16, height: u16, json: &str, state: &mut TreeState<Vec<Selector>>) -> Buffer {
        let json: Value = serde_json::from_str(json).expect("invalid test JSON");
        let tree = Tree::new(&json).highlight_symbol(">> ");
        let area = Rect::new(0, 0, width, height);
        let mut buffer = Buffer::empty(area);
        ratatui::widgets::StatefulWidget::render(tree, area, &mut buffer, state);
        buffer.set_style(area, Style::reset());
        buffer
    }

    #[test]
    fn empty_array_renders_nothing() {
        let buffer = render(5, 2, "[]", &mut TreeState::default());
        let expected = Buffer::with_lines(["  [] ", ""]);
        assert_eq!(buffer, expected);
    }

    #[test]
    fn empty_object_renders_nothing() {
        let buffer = render(5, 2, "{}", &mut TreeState::default());
        let expected = Buffer::with_lines(["  {} ", ""]);
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
        state.select(Some(vec![key("foo"), key("bar"), index(1)]));

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
