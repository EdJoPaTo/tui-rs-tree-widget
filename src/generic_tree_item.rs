use std::collections::HashSet;

use crate::{Node, TreeData};

/// Recursive generic tree item which can have children being of the same type.
///
/// Implements [`TreeData`] for `Vec<GenericTreeItem>`.
pub trait GenericTreeItem
where
    Self: Sized,
{
    /// Identifier of the current Item, which needs to be unique within the current depth.
    ///
    /// The relation of this Identifier to [`TreeData::Identifier`](crate::TreeData::Identifier) is
    /// `TreeData::Identifier = Vec<GenericTreeItem::Identifier>`.
    type Identifier: Clone + PartialEq + Eq + core::hash::Hash;

    #[must_use]
    fn identifier(&self) -> &Self::Identifier;

    #[must_use]
    fn children(&self) -> &[Self];

    #[must_use]
    fn height(&self) -> usize;

    fn render(&self, area: ratatui::layout::Rect, buffer: &mut ratatui::buffer::Buffer);

    #[must_use]
    fn child_direct<'root>(&'root self, identifier: &Self::Identifier) -> Option<&'root Self> {
        self.children()
            .iter()
            .find(|item| item.identifier() == identifier)
    }

    #[must_use]
    fn child_deep<'root>(&'root self, identifier: &[Self::Identifier]) -> Option<&'root Self> {
        let mut current = self;
        for identifier in identifier {
            current = current.child_direct(identifier)?;
        }
        Some(current)
    }
}

impl<Item: GenericTreeItem> TreeData for Vec<Item> {
    type Identifier = Vec<Item::Identifier>;

    fn get_nodes(
        &self,
        open_identifiers: &HashSet<Self::Identifier>,
    ) -> Vec<Node<Self::Identifier>> {
        flatten(open_identifiers, self, &[])
    }

    fn render(
        &self,
        identifier: &Self::Identifier,
        area: ratatui::layout::Rect,
        buffer: &mut ratatui::buffer::Buffer,
    ) {
        if let Some(item) = get_item(self, identifier) {
            item.render(area, buffer);
        };
    }
}

#[must_use]
pub fn get_item<'root, Item: GenericTreeItem>(
    root: &'root [Item],
    identifier: &[Item::Identifier],
) -> Option<&'root Item> {
    let mut identifier = identifier.iter();
    let initial_identifier = identifier.next()?;
    let mut current = root
        .iter()
        .find(|item| item.identifier() == initial_identifier)?;
    for identifier in identifier {
        current = current.child_direct(identifier)?;
    }
    Some(current)
}

#[must_use]
fn flatten<Item: GenericTreeItem>(
    open_identifiers: &HashSet<Vec<Item::Identifier>>,
    items: &[Item],
    current: &[Item::Identifier],
) -> Vec<Node<Vec<Item::Identifier>>> {
    let depth = current.len();
    let mut result = Vec::new();
    for item in items {
        let mut child_identifier = current.to_vec();
        child_identifier.push(item.identifier().clone());

        let children = item.children();
        let has_children = !children.is_empty();

        let child_result = open_identifiers
            .contains(&child_identifier)
            .then(|| flatten(open_identifiers, children, &child_identifier));

        result.push(Node {
            depth,
            has_children,
            height: item.height(),
            identifier: child_identifier,
        });

        if let Some(mut child_result) = child_result {
            result.append(&mut child_result);
        }
    }
    result
}

#[cfg(test)]
mod flatten_tests {
    use super::*;
    use crate::TreeItem;

    #[test]
    fn depth_works() {
        let mut open = HashSet::new();
        open.insert(vec!["b"]);
        open.insert(vec!["b", "d"]);
        let depths = flatten(&open, &TreeItem::example(), &[])
            .into_iter()
            .map(|flattened| flattened.depth)
            .collect::<Vec<_>>();
        assert_eq!(depths, [0, 0, 1, 1, 2, 2, 1, 0]);
    }

    fn case(open: &HashSet<Vec<&'static str>>, expected: &[&str]) {
        let items = TreeItem::example();
        let result = flatten(open, &items, &[]);
        let actual = result
            .into_iter()
            .map(|flattened| flattened.identifier.into_iter().last().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(actual, expected);
    }

    #[test]
    fn nothing_open_is_top_level() {
        let open = HashSet::new();
        case(&open, &["a", "b", "h"]);
    }

    #[test]
    fn wrong_open_is_only_top_level() {
        let mut open = HashSet::new();
        open.insert(vec!["a"]);
        open.insert(vec!["b", "d"]);
        case(&open, &["a", "b", "h"]);
    }

    #[test]
    fn one_is_open() {
        let mut open = HashSet::new();
        open.insert(vec!["b"]);
        case(&open, &["a", "b", "c", "d", "g", "h"]);
    }

    #[test]
    fn all_open() {
        let mut open = HashSet::new();
        open.insert(vec!["b"]);
        open.insert(vec!["b", "d"]);
        case(&open, &["a", "b", "c", "d", "e", "f", "g", "h"]);
    }
}
