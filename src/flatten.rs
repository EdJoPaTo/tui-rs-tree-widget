use std::collections::HashSet;

use crate::tree_item::TreeItem;

/// A flattened item of all visible [`TreeItem`]s.
///
/// Generated via [`TreeState::flatten`](crate::TreeState::flatten).
pub struct Flattened<'text, Identifier> {
    pub identifier: Vec<Identifier>,
    pub item: &'text TreeItem<'text, Identifier>,
}

impl<Identifier> Flattened<'_, Identifier> {
    /// Zero based depth. Depth 0 means top level with 0 indentation.
    #[must_use]
    pub fn depth(&self) -> usize {
        self.identifier.len() - 1
    }
}

/// Get a flat list of all visible [`TreeItem`]s.
///
/// `current` starts empty: `&[]`
#[must_use]
pub fn flatten<'text, Identifier>(
    open_identifiers: &HashSet<Vec<Identifier>>,
    items: &'text [TreeItem<'text, Identifier>],
    current: &[Identifier],
) -> Vec<Flattened<'text, Identifier>>
where
    Identifier: Clone + PartialEq + Eq + core::hash::Hash,
{
    let mut result = Vec::new();
    for item in items {
        let mut child_identifier = current.to_vec();
        child_identifier.push(item.identifier.clone());

        let child_result = open_identifiers
            .contains(&child_identifier)
            .then(|| flatten(open_identifiers, &item.children, &child_identifier));

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
fn depth_works() {
    let mut open = HashSet::new();
    open.insert(vec!["b"]);
    open.insert(vec!["b", "d"]);
    let depths = flatten(&open, &TreeItem::example(), &[])
        .into_iter()
        .map(|flattened| flattened.depth())
        .collect::<Vec<_>>();
    assert_eq!(depths, [0, 0, 1, 1, 2, 2, 1, 0]);
}

#[cfg(test)]
fn flatten_works(open: &HashSet<Vec<&'static str>>, expected: &[&str]) {
    let items = TreeItem::example();
    let result = flatten(open, &items, &[]);
    let actual = result
        .into_iter()
        .map(|flattened| flattened.identifier.into_iter().last().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(actual, expected);
}

#[test]
fn flatten_nothing_open_is_top_level() {
    let open = HashSet::new();
    flatten_works(&open, &["a", "b", "h"]);
}

#[test]
fn flatten_wrong_open_is_only_top_level() {
    let mut open = HashSet::new();
    open.insert(vec!["a"]);
    open.insert(vec!["b", "d"]);
    flatten_works(&open, &["a", "b", "h"]);
}

#[test]
fn flatten_one_is_open() {
    let mut open = HashSet::new();
    open.insert(vec!["b"]);
    flatten_works(&open, &["a", "b", "c", "d", "g", "h"]);
}

#[test]
fn flatten_all_open() {
    let mut open = HashSet::new();
    open.insert(vec!["b"]);
    open.insert(vec!["b", "d"]);
    flatten_works(&open, &["a", "b", "c", "d", "e", "f", "g", "h"]);
}
