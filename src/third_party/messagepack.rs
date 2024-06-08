/*! `MessagePack` implementation.
 *
 * While `MessagePack` seems key-value based at first it somewhat isnt as keys can be duplicated.
 * Therefore `KeyValueTreeItem` can not be implemented in a good way.
 * Falling back to implementing [`TreeData`].
 */

use std::borrow::Cow;
use std::collections::HashSet;

use ratatui::style::Style;
use ratatui::text::{Line, Span};
use rmpv::Value;

use crate::{Node, TreeData};

fn get_map_key_span(key: &Value) -> Span {
    use super::common::KEY;

    Span {
        content: match key {
            Value::Nil => Cow::Borrowed("nil"),
            Value::Boolean(true) => Cow::Borrowed("true"),
            Value::Boolean(false) => Cow::Borrowed("false"),
            Value::Integer(number) => Cow::Owned(number.to_string()),
            Value::F32(number) => Cow::Owned(number.to_string()),
            Value::F64(number) => Cow::Owned(number.to_string()),
            Value::String(string) if string.is_str() => Cow::Borrowed(string.as_str().unwrap()),
            Value::String(_) => Cow::Borrowed("non utf8 string"),
            Value::Binary(binary) => Cow::Owned(format!("{binary:?}")),
            Value::Array(array) if array.is_empty() => Cow::Borrowed("[]"),
            Value::Array(_) => Cow::Borrowed("[…]"),
            Value::Map(map) if map.is_empty() => Cow::Borrowed("{}"),
            Value::Map(_) => Cow::Borrowed("{…}"),
            Value::Ext(type_info, _) => Cow::Owned(format!("Ext({type_info}/…)")),
        },
        style: KEY,
    }
}

fn get_value_spans(value: &Value) -> Vec<Span> {
    use super::common::{BINARY, BOOL, ERROR, KEY, NULL, NUMBER, STRING};

    match value {
        Value::Array(array) if array.is_empty() => vec![Span {
            content: Cow::Borrowed("[]"),
            style: Style::new(),
        }],
        Value::Array(_) => vec![Span {
            content: Cow::Borrowed("["),
            style: Style::new(),
        }],
        Value::Map(map) if map.is_empty() => vec![Span {
            content: Cow::Borrowed("{}"),
            style: Style::new(),
        }],
        Value::Map(_) => vec![Span {
            content: Cow::Borrowed("{"),
            style: Style::new(),
        }],
        Value::Nil => vec![Span {
            content: Cow::Borrowed("nil"),
            style: NULL,
        }],
        Value::Boolean(true) => vec![Span {
            content: Cow::Borrowed("true"),
            style: BOOL,
        }],
        Value::Boolean(false) => vec![Span {
            content: Cow::Borrowed("false"),
            style: BOOL,
        }],
        Value::Integer(number) => vec![Span {
            content: Cow::Owned(number.to_string()),
            style: NUMBER,
        }],
        Value::F32(number) => vec![Span {
            content: Cow::Owned(number.to_string()),
            style: NUMBER,
        }],
        Value::F64(number) => vec![Span {
            content: Cow::Owned(number.to_string()),
            style: NUMBER,
        }],
        Value::String(string) if string.is_str() => vec![Span {
            content: Cow::Borrowed(string.as_str().unwrap()),
            style: STRING,
        }],
        Value::String(string) => vec![Span {
            content: Cow::Owned(string.as_err().unwrap().to_string()),
            style: ERROR,
        }],
        Value::Binary(binary) => vec![Span {
            content: Cow::Owned(format!("{binary:?}")),
            style: BINARY,
        }],
        Value::Ext(type_info, binary) => vec![
            Span {
                content: Cow::Owned(type_info.to_string()),
                style: KEY,
            },
            Span {
                content: Cow::Owned(format!("{binary:?}")),
                style: BINARY,
            },
        ],
    }
}

impl TreeData for Value {
    /// Index is the only correct key. Map is a `Vec<(K, V)>` and might contain duplicate keys.
    type Identifier = Vec<usize>;

    fn get_nodes(
        &self,
        open_identifiers: &HashSet<Self::Identifier>,
    ) -> Vec<Node<Self::Identifier>> {
        match self {
            Self::Array(array) if !array.is_empty() => flatten_array(open_identifiers, array, &[]),
            Self::Map(map) if !map.is_empty() => flatten_map(open_identifiers, map, &[]),
            _ => vec![Node {
                depth: 0,
                has_children: false,
                height: 1,
                identifier: Vec::new(),
            }],
        }
    }

    fn render(
        &self,
        identifier: &Self::Identifier,
        area: ratatui::layout::Rect,
        buffer: &mut ratatui::buffer::Buffer,
    ) {
        use super::common::{INDEX, NAME_SEPARATOR};

        if let Some((index, key, value)) = get_entry(self, identifier) {
            let mut spans = Vec::new();

            if let Some(index) = index {
                spans.push(Span {
                    content: Cow::Owned(index.to_string()),
                    style: INDEX,
                });
                spans.push(NAME_SEPARATOR);
            }

            if let Some(key) = key {
                spans.push(get_map_key_span(key));
                spans.push(NAME_SEPARATOR);
            }

            spans.append(&mut get_value_spans(value));

            let line = Line::from(spans);
            ratatui::widgets::Widget::render(line, area, buffer);
        }
    }
}

fn flatten_array(
    open_identifiers: &HashSet<Vec<usize>>,
    array: &[Value],
    current: &[usize],
) -> Vec<Node<Vec<usize>>> {
    let depth = current.len();
    let mut result = Vec::new();
    for (index, value) in array.iter().enumerate() {
        let mut child_identifier = current.to_vec();
        child_identifier.push(index);

        let child_result = if open_identifiers.contains(&child_identifier) {
            match value {
                Value::Array(array) => {
                    Some(flatten_array(open_identifiers, array, &child_identifier))
                }
                Value::Map(map) => Some(flatten_map(open_identifiers, map, &child_identifier)),
                _ => None,
            }
        } else {
            None
        };

        result.push(Node {
            depth,
            has_children: has_children(value),
            height: 1,
            identifier: child_identifier,
        });

        if let Some(mut child_result) = child_result {
            result.append(&mut child_result);
        }
    }
    result
}

fn flatten_map(
    open_identifiers: &HashSet<Vec<usize>>,
    map: &[(Value, Value)],
    current: &[usize],
) -> Vec<Node<Vec<usize>>> {
    let depth = current.len();
    let mut result = Vec::new();
    for (index, (_key, value)) in map.iter().enumerate() {
        let mut child_identifier = current.to_vec();
        child_identifier.push(index);

        let child_result = if open_identifiers.contains(&child_identifier) {
            match value {
                Value::Array(array) => {
                    Some(flatten_array(open_identifiers, array, &child_identifier))
                }
                Value::Map(map) => Some(flatten_map(open_identifiers, map, &child_identifier)),
                _ => None,
            }
        } else {
            None
        };

        result.push(Node {
            depth,
            has_children: has_children(value),
            height: 1,
            identifier: child_identifier,
        });

        if let Some(mut child_result) = child_result {
            result.append(&mut child_result);
        }
    }
    result
}

fn has_children(value: &Value) -> bool {
    match value {
        Value::Array(array) => !array.is_empty(),
        Value::Map(map) => !map.is_empty(),
        _ => false,
    }
}

fn get_entry<'root>(
    root: &'root Value,
    selector: &[usize],
) -> Option<(Option<usize>, Option<&'root Value>, &'root Value)> {
    let mut key = None;
    let mut current = root;
    for selector in selector {
        match current {
            Value::Array(array) => {
                current = array.get(*selector)?;
                key = None;
            }
            Value::Map(map) => {
                let element = map.get(*selector)?;
                key = Some(&element.0);
                current = &element.1;
            }
            _ => return None,
        }
    }
    let index = selector.last().copied();
    Some((index, key, current))
}

#[must_use]
pub fn get_value<'root>(root: &'root Value, selector: &[usize]) -> Option<&'root Value> {
    let mut current = root;
    for index in selector {
        match current {
            Value::Array(array) => current = array.get(*index)?,
            Value::Map(map) => current = &map.get(*index)?.1,
            _ => return None,
        }
    }
    Some(current)
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
    fn render(
        width: u16,
        height: u16,
        messagepack: &Value,
        state: &mut TreeState<Vec<usize>>,
    ) -> Buffer {
        let tree = Tree::new(messagepack).highlight_symbol(">> ");
        let area = Rect::new(0, 0, width, height);
        let mut buffer = Buffer::empty(area);
        ratatui::widgets::StatefulWidget::render(tree, area, &mut buffer, state);
        buffer.set_style(area, Style::reset());
        buffer
    }

    #[test]
    fn empty_array() {
        let input = Value::Array(Vec::new());
        let buffer = render(5, 2, &input, &mut TreeState::default());
        let expected = Buffer::with_lines(["  [] ", ""]);
        assert_eq!(buffer, expected);
    }

    #[test]
    fn empty_map() {
        let input = Value::Map(Vec::new());
        let buffer = render(5, 2, &input, &mut TreeState::default());
        let expected = Buffer::with_lines(["  {} ", ""]);
        assert_eq!(buffer, expected);
    }

    #[test]
    fn number() {
        let input = Value::Integer(42.into());
        let buffer = render(5, 2, &input, &mut TreeState::default());
        let expected = Buffer::with_lines(["  42 ", ""]);
        assert_eq!(buffer, expected);
    }

    #[test]
    fn all_simple_in_array() {
        let input = Value::Array(vec![
            Value::Nil,
            Value::Boolean(true),
            Value::Boolean(false),
            Value::Array(Vec::new()),
            Value::Map(Vec::new()),
            Value::Integer(42.into()),
            Value::String("lalala".into()),
        ]);
        let buffer = render(12, 8, &input, &mut TreeState::default());
        let expected = Buffer::with_lines([
            "  0: nil    ",
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

    fn get_bigger_example() -> Value {
        Value::Map(vec![
            (
                Value::String("foo".into()),
                Value::Map(vec![(
                    Value::String("bar".into()),
                    Value::Array(vec![Value::Integer(13.into()), Value::Integer(37.into())]),
                )]),
            ),
            (Value::String("test".into()), Value::Boolean(true)),
        ])
    }

    #[test]
    fn bigger_example() {
        let mut state = TreeState::default();
        state.open(vec![0]);
        state.open(vec![0, 0]);

        let buffer = render(17, 6, &get_bigger_example(), &mut state);
        let expected = Buffer::with_lines([
            "▼ 0: foo: {      ",
            "  ▼ 0: bar: [    ",
            "      0: 13      ",
            "      1: 37      ",
            "  1: test: true  ",
            "                 ",
        ]);
        assert_eq!(buffer, expected);
    }

    #[test]
    fn bigger_example_selection() {
        let mut state = TreeState::default();
        state.open(vec![0]);
        state.open(vec![0, 0]);
        state.select(Some(vec![0, 0, 1]));

        let buffer = render(20, 6, &get_bigger_example(), &mut state);
        let expected = Buffer::with_lines([
            "   ▼ 0: foo: {      ",
            "     ▼ 0: bar: [    ",
            "         0: 13      ",
            ">>       1: 37      ",
            "     1: test: true  ",
            "                    ",
        ]);
        assert_eq!(buffer, expected);
    }
}
