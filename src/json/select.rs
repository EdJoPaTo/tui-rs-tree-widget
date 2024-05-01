use serde_json::Value;

use crate::identifier::Selector;

/// Select one layer into `root` (depth == 1).
fn select_one<'value>(root: &'value Value, selector: &Selector) -> Option<&'value Value> {
    match (root, selector) {
        (Value::Object(object), Selector::ObjectKey(key)) => object.get(key),
        (Value::Array(array), Selector::ArrayIndex(index)) => array.get(*index),
        (Value::Bool(_) | Value::Null | Value::Number(_) | Value::String(_), Selector::None) => {
            Some(root)
        }
        _ => None,
    }
}

/// Select a part of the input [JSON](Value).
#[must_use]
pub fn select<'value>(root: &'value Value, selector: &[Selector]) -> Option<&'value Value> {
    let mut current = root;
    for select in selector {
        current = select_one(current, select)?;
    }
    Some(current)
}

#[test]
fn can_not_get_other_value() {
    let root = Value::Bool(false);
    let result = select_one(&root, &Selector::ArrayIndex(2));
    assert_eq!(result, None);
}

#[test]
fn can_get_nth_array_value() {
    let root = Value::Array(vec![Value::String("bla".to_owned()), Value::Bool(true)]);
    let result = select_one(&root, &Selector::ArrayIndex(1));
    assert_eq!(result, Some(&Value::Bool(true)));
}

#[test]
fn can_not_get_array_index_out_of_range() {
    let root = Value::Array(vec![Value::String("bla".to_owned()), Value::Bool(true)]);
    let result = select_one(&root, &Selector::ArrayIndex(42));
    assert_eq!(result, None);
}

#[test]
fn can_get_object_value() {
    let mut object = serde_json::Map::new();
    object.insert("bla".to_owned(), Value::Bool(false));
    object.insert("blubb".to_owned(), Value::Bool(true));
    let root = Value::Object(object);
    let result = select_one(&root, &Selector::ObjectKey("blubb".to_owned()));
    assert_eq!(result, Some(&Value::Bool(true)));
}

#[test]
fn can_not_get_object_missing_key() {
    let mut object = serde_json::Map::new();
    object.insert("bla".to_owned(), Value::Bool(false));
    object.insert("blubb".to_owned(), Value::Bool(true));
    let root = Value::Object(object);
    let result = select_one(&root, &Selector::ObjectKey("foo".to_owned()));
    assert_eq!(result, None);
}

#[test]
fn can_not_get_object_by_index() {
    let mut object = serde_json::Map::new();
    object.insert("bla".to_owned(), Value::Bool(false));
    object.insert("blubb".to_owned(), Value::Bool(true));
    let root = Value::Object(object);
    let result = select_one(&root, &Selector::ArrayIndex(42));
    assert_eq!(result, None);
}

#[test]
fn can_get_selected_value() {
    let mut inner = serde_json::Map::new();
    inner.insert("bla".to_owned(), Value::Bool(false));
    inner.insert("blubb".to_owned(), Value::Bool(true));

    let root = Value::Array(vec![
        Value::Bool(false),
        Value::Object(inner),
        Value::Bool(false),
    ]);

    let selector = vec![
        Selector::ArrayIndex(1),
        Selector::ObjectKey("blubb".to_owned()),
    ];

    let result = select(&root, &selector);
    assert_eq!(result, Some(&Value::Bool(true)));
}
