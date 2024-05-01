use std::collections::HashSet;

use ratatui::style::Style;
use ratatui::text::Text;

use crate::tree_item::TreeItem;

/// A flattened item of all visible [`TreeItem`]s.
///
/// Generated via [`TreeState::flatten`](crate::TreeState::flatten).
pub struct Flattened<'a, Identifier> {
    pub identifier: Vec<Identifier>,

    pub has_no_children: bool,
    pub height: usize,
    pub text: Text<'a>,
    pub style: Style,
}

impl<'a, Identifier> Flattened<'a, Identifier> {
    #[must_use]
    pub fn depth(&self) -> usize {
        self.identifier.len() - 1
    }
}

/// Get a flat list of all visible [`TreeItem`]s.
///
/// `current` starts empty: `&[]`
#[must_use]
pub fn flatten<'a, Identifier>(
    opened: &HashSet<Vec<Identifier>>,
    items: Vec<TreeItem<'a, Identifier>>,
    current: &[Identifier],
) -> Vec<Flattened<'a, Identifier>>
where
    Identifier: Clone + PartialEq + Eq + core::hash::Hash,
{
    let mut result = Vec::new();
    for item in items {
        let mut child_identifier = current.to_vec();
        child_identifier.push(item.identifier.clone());

        let has_no_children = item.children.is_empty();

        let child_result = opened
            .contains(&child_identifier)
            .then(|| flatten(opened, item.children, &child_identifier));

        result.push(Flattened {
            identifier: child_identifier,

            has_no_children,
            height: item.text.height(),
            style: item.style,
            text: item.text,
        });

        if let Some(mut child_result) = child_result {
            result.append(&mut child_result);
        }
    }
    result
}

/// Height requireed to show all visible/opened items.
///
/// `current` starts empty: `&[]`
pub fn total_required_height<Identifier>(
    opened: &HashSet<Vec<Identifier>>,
    items: &[TreeItem<'_, Identifier>],
    current: &[Identifier],
) -> usize
where
    Identifier: Clone + PartialEq + Eq + core::hash::Hash,
{
    let mut result: usize = 0;
    for item in items {
        let mut child_identifier = current.to_vec();
        child_identifier.push(item.identifier.clone());

        result = result.saturating_add(item.text.height());

        if opened.contains(&child_identifier) {
            let below = total_required_height(opened, &item.children, &child_identifier);
            result = result.saturating_add(below);
        }
    }
    result
}

#[cfg(test)]
fn flatten_works(opened: &HashSet<Vec<&'static str>>, expected: &[&str]) {
    let result = flatten(opened, TreeItem::example(), &[]);
    let actual = result
        .into_iter()
        .map(|flattened| flattened.identifier.into_iter().last().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(actual, expected);
}

#[test]
fn get_opened_nothing_opened_is_top_level() {
    let opened = HashSet::new();
    flatten_works(&opened, &["a", "b", "h"]);
}

#[test]
fn get_opened_wrong_opened_is_only_top_level() {
    let mut opened = HashSet::new();
    opened.insert(vec!["a"]);
    opened.insert(vec!["b", "d"]);
    flatten_works(&opened, &["a", "b", "h"]);
}

#[test]
fn get_opened_one_is_opened() {
    let mut opened = HashSet::new();
    opened.insert(vec!["b"]);
    flatten_works(&opened, &["a", "b", "c", "d", "g", "h"]);
}

#[test]
fn get_opened_all_opened() {
    let mut opened = HashSet::new();
    opened.insert(vec!["b"]);
    opened.insert(vec!["b", "d"]);
    flatten_works(&opened, &["a", "b", "c", "d", "e", "f", "g", "h"]);
}

#[cfg(test)]
#[track_caller]
fn required_height_works(opened: &HashSet<Vec<&'static str>>, expected: usize) {
    let actual = total_required_height(opened, &TreeItem::example(), &[]);
    assert_eq!(actual, expected);
}

#[test]
fn nothing_opened_height() {
    let opened = HashSet::new();
    required_height_works(&opened, 3);
}

#[test]
fn wrong_opened_height() {
    let mut opened = HashSet::new();
    opened.insert(vec!["a"]);
    opened.insert(vec!["b", "d"]);
    required_height_works(&opened, 3);
}

#[test]
fn opened_one_height() {
    let mut opened = HashSet::new();
    opened.insert(vec!["b"]);
    required_height_works(&opened, 6);
}

#[test]
fn opened_all_height() {
    let mut opened = HashSet::new();
    opened.insert(vec!["b"]);
    opened.insert(vec!["b", "d"]);
    required_height_works(&opened, 8);
}
