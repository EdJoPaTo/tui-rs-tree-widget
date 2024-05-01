use std::collections::HashSet;

use crate::tree_item::TreeItem;

/// A flattened item of all visible [`TreeItem`]s.
///
/// Generated via [`TreeState::flatten`](crate::TreeState::flatten).
pub struct Flattened<'a, Identifier> {
    pub identifier: Vec<Identifier>,
    pub item: &'a TreeItem<'a, Identifier>,
}

impl<'a, Identifier> Flattened<'a, Identifier> {
    #[must_use]
    pub fn depth(&self) -> usize {
        self.identifier.len() - 1
    }
}

/// Get a flat list of all visible [`TreeItem`]s.
#[must_use]
pub fn flatten<'a, Identifier>(
    opened: &HashSet<Vec<Identifier>>,
    items: &'a [TreeItem<'a, Identifier>],
) -> Vec<Flattened<'a, Identifier>>
where
    Identifier: Clone + PartialEq + Eq + core::hash::Hash,
{
    internal(opened, items, &[])
}

#[must_use]
fn internal<'a, Identifier>(
    opened: &HashSet<Vec<Identifier>>,
    items: &'a [TreeItem<'a, Identifier>],
    current: &[Identifier],
) -> Vec<Flattened<'a, Identifier>>
where
    Identifier: Clone + PartialEq + Eq + core::hash::Hash,
{
    let mut result = Vec::new();
    for item in items {
        let mut child_identifier = current.to_vec();
        child_identifier.push(item.identifier.clone());

        let child_result = opened
            .contains(&child_identifier)
            .then(|| internal(opened, &item.children, &child_identifier));

        result.push(Flattened {
            identifier: child_identifier,
            item,
        });

        if let Some(mut child_result) = child_result {
            result.append(&mut child_result);
        }
    }
    result
}

#[test]
fn get_opened_nothing_opened_is_top_level() {
    let items = TreeItem::example();
    let opened = HashSet::new();
    let result = flatten(&opened, &items);
    let result_text = result
        .into_iter()
        .map(|flattened| flattened.item.identifier)
        .collect::<Vec<_>>();
    assert_eq!(result_text, ["a", "b", "h"]);
}

#[test]
fn get_opened_wrong_opened_is_only_top_level() {
    let items = TreeItem::example();
    let mut opened = HashSet::new();
    opened.insert(vec!["a"]);
    opened.insert(vec!["b", "d"]);
    let result = flatten(&opened, &items);
    let result_text = result
        .into_iter()
        .map(|flattened| flattened.item.identifier)
        .collect::<Vec<_>>();
    assert_eq!(result_text, ["a", "b", "h"]);
}

#[test]
fn get_opened_one_is_opened() {
    let items = TreeItem::example();
    let mut opened = HashSet::new();
    opened.insert(vec!["b"]);
    let result = flatten(&opened, &items);
    let result_text = result
        .into_iter()
        .map(|flattened| flattened.item.identifier)
        .collect::<Vec<_>>();
    assert_eq!(result_text, ["a", "b", "c", "d", "g", "h"]);
}

#[test]
fn get_opened_all_opened() {
    let items = TreeItem::example();
    let mut opened = HashSet::new();
    opened.insert(vec!["b"]);
    opened.insert(vec!["b", "d"]);
    let result = flatten(&opened, &items);
    let result_text = result
        .into_iter()
        .map(|flattened| flattened.item.identifier)
        .collect::<Vec<_>>();
    assert_eq!(result_text, ["a", "b", "c", "d", "e", "f", "g", "h"]);
}
