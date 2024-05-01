use std::collections::HashSet;

use crate::tree_item::TreeItem;

/// A flattened item of all visible [`TreeItem`]s.
///
/// Generated via [`TreeState::flatten`](crate::TreeState::flatten).
pub struct Flattened<'item, Item, Identifier> {
    pub identifier: Vec<Identifier>,
    pub item: &'item Item,

    pub has_no_children: bool,
    pub height: usize,
}

impl<Item, Identifier> Flattened<'_, Item, Identifier> {
    /// Zero based depth. Depth 0 means 0 indentation.
    #[must_use]
    pub fn depth(&self) -> usize {
        self.identifier.len() - 1
    }
}

/// Get a flat list of all visible [`TreeItem`]s.
///
/// `current` starts empty: `&[]`
#[must_use]
pub fn flatten<'item, Item>(
    opened: &HashSet<Vec<Item::Identifier>>,
    items: &'item [Item],
    current: &[Item::Identifier],
) -> Vec<Flattened<'item, Item, Item::Identifier>>
where
    Item: TreeItem,
{
    let mut result = Vec::new();
    for item in items {
        let mut child_identifier = current.to_vec();
        child_identifier.push(item.identifier().clone());

        let children = item.children();
        let has_no_children = children.is_empty();

        let child_result = opened
            .contains(&child_identifier)
            .then(|| flatten(opened, children, &child_identifier));

        result.push(Flattened {
            identifier: child_identifier,
            item,

            has_no_children,
            height: item.height(),
        });

        if let Some(mut child_result) = child_result {
            result.append(&mut child_result);
        }
    }
    result
}

/// Height required to show all visible/opened items.
///
/// `current` starts empty: `&[]`
pub fn total_required_height<Item>(
    opened: &HashSet<Vec<Item::Identifier>>,
    items: &[Item],
    current: &[Item::Identifier],
) -> usize
where
    Item: TreeItem,
{
    let mut result: usize = 0;
    for item in items {
        let mut child_identifier = current.to_vec();
        child_identifier.push(item.identifier().clone());

        result = result.saturating_add(item.height());

        if opened.contains(&child_identifier) {
            let below = total_required_height(opened, item.children(), &child_identifier);
            result = result.saturating_add(below);
        }
    }
    result
}

#[test]
fn depth_works() {
    let mut opened = HashSet::new();
    opened.insert(vec!["b"]);
    opened.insert(vec!["b", "d"]);
    let depths = flatten(&opened, &crate::SimpleTreeItem::example(), &[])
        .into_iter()
        .map(|flattened| flattened.depth())
        .collect::<Vec<_>>();
    assert_eq!(depths, [0, 0, 1, 1, 2, 2, 1, 0]);
}

#[cfg(test)]
fn flatten_works(opened: &HashSet<Vec<&'static str>>, expected: &[&str]) {
    let items = crate::SimpleTreeItem::example();
    let result = flatten(opened, &items, &[]);
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
    let actual = total_required_height(opened, &crate::SimpleTreeItem::example(), &[]);
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
