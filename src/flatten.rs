use std::collections::HashSet;

use crate::item::Item;

/// A flattened item of all visible [`TreeItem`s](Item).
///
/// Generated via [`TreeState::flatten`](crate::TreeState::flatten).
pub struct Flattened<'a, Identifier> {
    pub identifier: Vec<Identifier>,
    pub item: &'a Item<'a, Identifier>,
}

impl<'a, Identifier> Flattened<'a, Identifier> {
    #[must_use]
    pub fn depth(&self) -> usize {
        self.identifier.len() - 1
    }
}

/// Get a flat list of all visible [`TreeItem`s](TreeItem).
#[must_use]
pub fn flatten<'a, Identifier>(
    opened: &HashSet<Vec<Identifier>>,
    items: &'a [Item<'a, Identifier>],
) -> Vec<Flattened<'a, Identifier>>
where
    Identifier: Clone + PartialEq + Eq + core::hash::Hash,
{
    internal(opened, items, &[])
}

#[must_use]
fn internal<'a, Identifier>(
    opened: &HashSet<Vec<Identifier>>,
    items: &'a [Item<'a, Identifier>],
    current: &[Identifier],
) -> Vec<Flattened<'a, Identifier>>
where
    Identifier: Clone + PartialEq + Eq + core::hash::Hash,
{
    let mut result = Vec::new();
    for item in items {
        let mut child_identifier = current.to_vec();
        child_identifier.push(item.identifier.clone());

        result.push(Flattened {
            item,
            identifier: child_identifier.clone(),
        });

        if opened.contains(&child_identifier) {
            let mut child_result = internal(opened, &item.children, &child_identifier);
            result.append(&mut child_result);
        }
    }
    result
}

#[cfg(test)]
fn get_example_tree_items() -> Vec<Item<'static, &'static str>> {
    vec![
        Item::new_leaf("a", "Alfa"),
        Item::new(
            "b",
            "Bravo",
            vec![
                Item::new_leaf("c", "Charlie"),
                Item::new(
                    "d",
                    "Delta",
                    vec![Item::new_leaf("e", "Echo"), Item::new_leaf("f", "Foxtrot")],
                )
                .expect("all item identifiers are unique"),
                Item::new_leaf("g", "Golf"),
            ],
        )
        .expect("all item identifiers are unique"),
        Item::new_leaf("h", "Hotel"),
    ]
}

#[test]
fn get_opened_nothing_opened_is_top_level() {
    let items = get_example_tree_items();
    let opened = HashSet::new();
    let result = flatten(&opened, &items);
    let result_text = result.iter().map(|o| o.item.identifier).collect::<Vec<_>>();
    assert_eq!(result_text, ["a", "b", "h"]);
}

#[test]
fn get_opened_wrong_opened_is_only_top_level() {
    let items = get_example_tree_items();
    let mut opened = HashSet::new();
    opened.insert(vec!["a"]);
    opened.insert(vec!["b", "d"]);
    let result = flatten(&opened, &items);
    let result_text = result.iter().map(|o| o.item.identifier).collect::<Vec<_>>();
    assert_eq!(result_text, ["a", "b", "h"]);
}

#[test]
fn get_opened_one_is_opened() {
    let items = get_example_tree_items();
    let mut opened = HashSet::new();
    opened.insert(vec!["b"]);
    let result = flatten(&opened, &items);
    let result_text = result.iter().map(|o| o.item.identifier).collect::<Vec<_>>();
    assert_eq!(result_text, ["a", "b", "c", "d", "g", "h"]);
}

#[test]
fn get_opened_all_opened() {
    let items = get_example_tree_items();
    let mut opened = HashSet::new();
    opened.insert(vec!["b"]);
    opened.insert(vec!["b", "d"]);
    let result = flatten(&opened, &items);
    let result_text = result.iter().map(|o| o.item.identifier).collect::<Vec<_>>();
    assert_eq!(result_text, ["a", "b", "c", "d", "e", "f", "g", "h"]);
}
