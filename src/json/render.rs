use std::borrow::Cow;
use std::collections::HashSet;

use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use serde_json::Value;

use super::select;
use crate::identifier::Selector;
use crate::{Node, TreeData};

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

impl TreeData for Value {
    type Identifier = Selector;

    fn get_nodes(
        &self,
        open_identifiers: &HashSet<Vec<Self::Identifier>>,
    ) -> Vec<Node<Self::Identifier>> {
        match self {
            Self::Null | Self::Bool(_) | Self::Number(_) | Self::String(_) => vec![Node {
                identifier: vec![Selector::None],
                has_children: false,
                height: 1,
            }],
            Self::Array(array) => get_array_nodes(open_identifiers, array, &[]),
            Self::Object(object) => get_object_nodes(open_identifiers, object, &[]),
        }
    }

    fn render(
        &self,
        identifier: &[Self::Identifier],
        area: ratatui::layout::Rect,
        buffer: &mut ratatui::buffer::Buffer,
    ) {
        const KEY: Style = Style::new().fg(Color::Blue);
        const INDEX: Style = Style::new().fg(Color::Cyan);

        const NAME_SEPARATOR: Span = Span {
            content: Cow::Borrowed(": "),
            style: Style::new().fg(Color::DarkGray),
        };

        let Some(key) = identifier.last() else {
            return;
        };
        let Some(value) = select(self, identifier) else {
            return;
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
        ratatui::widgets::Widget::render(&text, area, buffer);
    }
}

fn get_nodes_recursive(
    open_identifiers: &HashSet<Vec<Selector>>,
    json: &Value,
    current_identifier: Vec<Selector>,
) -> Vec<Node<Selector>> {
    match json {
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => vec![Node {
            identifier: current_identifier,
            has_children: false,
            height: 1,
        }],
        Value::Array(array) => {
            let mut result = Vec::new();
            let children = open_identifiers
                .contains(&current_identifier)
                .then(|| get_array_nodes(open_identifiers, array, &current_identifier));
            result.push(Node {
                identifier: current_identifier,
                has_children: !array.is_empty(),
                height: 1,
            });
            if let Some(mut children) = children {
                result.append(&mut children);
            }
            result
        }
        Value::Object(object) => {
            let mut result = Vec::new();
            let children = open_identifiers
                .contains(&current_identifier)
                .then(|| get_object_nodes(open_identifiers, object, &current_identifier));
            result.push(Node {
                identifier: current_identifier,
                has_children: !object.is_empty(),
                height: 1,
            });
            if let Some(mut children) = children {
                result.append(&mut children);
            }
            result
        }
    }
}

fn get_object_nodes(
    open_identifiers: &HashSet<Vec<Selector>>,
    object: &serde_json::Map<String, Value>,
    current_identifier: &[Selector],
) -> Vec<Node<Selector>> {
    object
        .iter()
        .flat_map(|(key, value)| {
            let mut child_identifier = current_identifier.to_vec();
            child_identifier.push(Selector::ObjectKey(key.clone()));
            get_nodes_recursive(open_identifiers, value, child_identifier)
        })
        .collect()
}

fn get_array_nodes(
    open_identifiers: &HashSet<Vec<Selector>>,
    array: &[Value],
    current_identifier: &[Selector],
) -> Vec<Node<Selector>> {
    array
        .iter()
        .enumerate()
        .flat_map(|(index, value)| {
            let mut child_identifier = current_identifier.to_vec();
            child_identifier.push(Selector::ArrayIndex(index));
            get_nodes_recursive(open_identifiers, value, child_identifier)
        })
        .collect()
}

#[cfg(test)]
fn key(key: &str) -> Selector {
    Selector::ObjectKey(key.to_owned())
}

#[cfg(test)]
mod tree_data_tests {
    use super::*;

    const fn node(identifier: Vec<Selector>, has_children: bool) -> Node<Selector> {
        Node {
            identifier,
            has_children,
            height: 1,
        }
    }

    #[track_caller]
    fn case(input: &str) -> Vec<Node<Selector>> {
        let mut open = HashSet::new();
        open.insert(vec![key("foo")]);
        open.insert(vec![key("foo"), key("bar")]);

        let json: Value = serde_json::from_str(input).expect("invalid JSON string");
        json.get_nodes(&open)
    }

    #[test]
    fn empty_array_has_no_nodes() {
        assert_eq!(case("[]"), []);
    }

    #[test]
    fn empty_object_has_no_nodes() {
        assert_eq!(case("{}"), []);
    }

    #[test]
    fn number_has_single_node() {
        assert_eq!(case("42"), [node(vec![Selector::None], false)]);
    }

    #[test]
    fn root_array_has_multiple_nodes() {
        assert_eq!(
            case("[13, 37]"),
            [
                node(vec![Selector::ArrayIndex(0)], false),
                node(vec![Selector::ArrayIndex(1)], false),
            ]
        );
    }

    #[test]
    fn root_object_has_multiple_nodes() {
        assert_eq!(
            case(r#"{"foo": "bar", "something": true}"#),
            [
                node(vec![key("foo")], false),
                node(vec![key("something")], false),
            ]
        );
    }

    #[test]
    fn deep_example() {
        assert_eq!(
            case(r#"{"foo": {"bar": [13, 37]}, "something": [42]}"#),
            [
                node(vec![key("foo")], true),             // open
                node(vec![key("foo"), key("bar")], true), // open
                node(vec![key("foo"), key("bar"), Selector::ArrayIndex(0)], false),
                node(vec![key("foo"), key("bar"), Selector::ArrayIndex(1)], false),
                node(vec![key("something")], true),
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
    fn render(width: u16, height: u16, json: &str, state: &mut TreeState<Selector>) -> Buffer {
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
