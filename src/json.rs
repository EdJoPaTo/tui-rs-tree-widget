use std::borrow::Cow;
use std::collections::HashSet;

use jsonptr::{Pointer, Token};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use serde_json::Value;

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
    type Identifier = Pointer;

    fn get_nodes(
        &self,
        open_identifiers: &HashSet<Self::Identifier>,
    ) -> Vec<Node<Self::Identifier>> {
        match self {
            Self::Null | Self::Bool(_) | Self::Number(_) | Self::String(_) => vec![Node {
                depth: 0,
                has_children: false,
                height: 1,
                identifier: Pointer::root(),
            }],
            Self::Array(array) => get_array_nodes(open_identifiers, array, &Pointer::root()),
            Self::Object(object) => get_object_nodes(open_identifiers, object, &Pointer::root()),
        }
    }

    fn render(
        &self,
        identifier: &Self::Identifier,
        area: ratatui::layout::Rect,
        buffer: &mut ratatui::buffer::Buffer,
    ) {
        const KEY: Style = Style::new().fg(Color::Blue);
        const INDEX: Style = Style::new().fg(Color::Cyan);

        const NAME_SEPARATOR: Span = Span {
            content: Cow::Borrowed(": "),
            style: Style::new().fg(Color::DarkGray),
        };

        let Ok(value) = identifier.resolve(self) else {
            return;
        };

        let mut parent = identifier.clone();
        parent.pop_back();
        let parent_is_array = parent.resolve(self).is_ok_and(Self::is_array);

        let value_span = get_value_span(value);
        let spans = if let Some(key) = identifier.last() {
            vec![
                Span {
                    content: Cow::Owned(key.as_key().to_owned()),
                    style: if parent_is_array { INDEX } else { KEY },
                },
                NAME_SEPARATOR,
                value_span,
            ]
        } else {
            vec![value_span]
        };
        let text = Line::from(spans);
        ratatui::widgets::Widget::render(&text, area, buffer);
    }
}

fn get_nodes_recursive(
    open_identifiers: &HashSet<Pointer>,
    json: &Value,
    current_identifier: Pointer,
) -> Vec<Node<Pointer>> {
    let depth = current_identifier.count() - 1;
    match json {
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => vec![Node {
            depth,
            has_children: false,
            height: 1,
            identifier: current_identifier,
        }],
        Value::Array(array) => {
            let mut result = Vec::new();
            let children = open_identifiers
                .contains(&current_identifier)
                .then(|| get_array_nodes(open_identifiers, array, &current_identifier));
            result.push(Node {
                depth,
                has_children: !array.is_empty(),
                height: 1,
                identifier: current_identifier,
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
                depth,
                has_children: !object.is_empty(),
                height: 1,
                identifier: current_identifier,
            });
            if let Some(mut children) = children {
                result.append(&mut children);
            }
            result
        }
    }
}

fn get_object_nodes(
    open_identifiers: &HashSet<Pointer>,
    object: &serde_json::Map<String, Value>,
    current_identifier: &Pointer,
) -> Vec<Node<Pointer>> {
    object
        .iter()
        .flat_map(|(key, value)| {
            let mut child_identifier = current_identifier.clone();
            child_identifier.push_back(Token::new(key));
            get_nodes_recursive(open_identifiers, value, child_identifier)
        })
        .collect()
}

fn get_array_nodes(
    open_identifiers: &HashSet<Pointer>,
    array: &[Value],
    current_identifier: &Pointer,
) -> Vec<Node<Pointer>> {
    array
        .iter()
        .enumerate()
        .flat_map(|(index, value)| {
            let mut child_identifier = current_identifier.clone();
            child_identifier.push_back(Token::new(index.to_string()));
            get_nodes_recursive(open_identifiers, value, child_identifier)
        })
        .collect()
}

#[cfg(test)]
#[track_caller]
fn pointer(value: &str) -> Pointer {
    Pointer::parse(value).unwrap()
}

#[cfg(test)]
mod tree_data_tests {
    use super::*;

    #[track_caller]
    fn node(pointer: &str, has_children: bool) -> Node<Pointer> {
        let pointer = Pointer::parse(pointer).unwrap();
        Node {
            depth: pointer.count().saturating_sub(1),
            has_children,
            height: 1,
            identifier: pointer,
        }
    }

    #[test]
    fn node_helper_works() {
        let result = node("/foo/bar", false);
        let expected = Node {
            depth: 1,
            has_children: false,
            height: 1,
            identifier: Pointer::new(["foo", "bar"]),
        };
        assert_eq!(result, expected);
    }

    #[track_caller]
    fn case(input: &str) -> Vec<Node<Pointer>> {
        let mut open = HashSet::new();
        open.insert(Pointer::parse("/foo").unwrap());
        open.insert(Pointer::parse("/foo/bar").unwrap());

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
        assert_eq!(case("42"), [node("", false)]);
    }

    #[test]
    fn root_array_has_multiple_nodes() {
        assert_eq!(case("[13, 37]"), [node("/0", false), node("/1", false),]);
    }

    #[test]
    fn root_object_has_multiple_nodes() {
        assert_eq!(
            case(r#"{"foo": "bar", "something": true}"#),
            [node("/foo", false), node("/something", false),]
        );
    }

    #[test]
    fn deep_example() {
        assert_eq!(
            case(r#"{"foo": {"bar": [13, 37]}, "something": [42]}"#),
            [
                node("/foo", true),     // open
                node("/foo/bar", true), // open
                node("/foo/bar/0", false),
                node("/foo/bar/1", false),
                node("/something", true),
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
    fn render(width: u16, height: u16, json: &str, state: &mut TreeState<Pointer>) -> Buffer {
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
        state.open(pointer("/foo"));
        state.open(pointer("/foo/bar"));

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
        state.open(pointer("/foo"));
        state.open(pointer("/foo/bar"));
        state.select(Some(pointer("/foo/bar/1")));

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
