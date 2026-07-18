use std::collections::HashSet;

use crate::tree_item::TreeItem;

/// A flattened item of all visible [`TreeItem`]s.
///
/// Generated via [`TreeState::flatten`](crate::TreeState::flatten).
#[must_use]
pub struct Flattened<'text, Identifier> {
    pub identifier: Vec<Identifier>,
    /// For each depth level of this row's path (index 0 is the top level),
    /// whether the node at that level has a following sibling.
    /// Length equals `identifier.len()`.
    ///
    /// This drives indent guide rendering: a level with a following sibling
    /// needs a continuing vertical line (`│`), the deepest level picks
    /// between a branch (`├`) and a last-branch (`└`) connector.
    pub has_next_sibling: Vec<bool>,
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
/// `current` and `current_siblings` start empty: `&[]`
#[must_use]
pub fn flatten<'text, Identifier>(
    open_identifiers: &HashSet<Vec<Identifier>>,
    items: &'text [TreeItem<'text, Identifier>],
    current: &[Identifier],
    current_siblings: &[bool],
) -> Vec<Flattened<'text, Identifier>>
where
    Identifier: Clone + PartialEq + Eq + core::hash::Hash,
{
    let mut result = Vec::new();
    let last_index = items.len().saturating_sub(1);
    for (index, item) in items.iter().enumerate() {
        let mut child_identifier = current.to_vec();
        child_identifier.push(item.identifier.clone());

        let mut child_siblings = current_siblings.to_vec();
        child_siblings.push(index != last_index);

        let child_result = open_identifiers.contains(&child_identifier).then(|| {
            flatten(
                open_identifiers,
                &item.children,
                &child_identifier,
                &child_siblings,
            )
        });

        result.push(Flattened {
            identifier: child_identifier,
            has_next_sibling: child_siblings,
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
    let depths = flatten(&open, &TreeItem::example(), &[], &[])
        .into_iter()
        .map(|flattened| flattened.depth())
        .collect::<Vec<_>>();
    assert_eq!(depths, [0, 0, 1, 1, 2, 2, 1, 0]);
}

#[test]
fn has_next_sibling_works() {
    let mut open = HashSet::new();
    open.insert(vec!["b"]);
    open.insert(vec!["b", "d"]);
    let actual = flatten(&open, &TreeItem::example(), &[], &[])
        .into_iter()
        .map(|flattened| flattened.has_next_sibling)
        .collect::<Vec<_>>();
    // Order: a, b, b/c, b/d, b/d/e, b/d/f, b/g, h
    assert_eq!(
        actual,
        [
            vec![true],              // a  (b, h follow)
            vec![true],              // b  (h follows)
            vec![true, true],        // b/c (b continues; d, g follow c)
            vec![true, true],        // b/d (b continues; g follows d)
            vec![true, true, true],  // b/d/e (b, d continue; f follows e)
            vec![true, true, false], // b/d/f (b, d continue; f is last)
            vec![true, false],       // b/g (b continues; g is last child of b)
            vec![false],             // h  (last top level item)
        ]
    );
}

#[cfg(test)]
fn flatten_works(open: &HashSet<Vec<&'static str>>, expected: &[&str]) {
    let items = TreeItem::example();
    let result = flatten(open, &items, &[], &[]);
    let actual = result
        .into_iter()
        .map(|flattened| flattened.identifier.into_iter().next_back().unwrap())
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
