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
}

#[must_use]
fn get_item<'root, Item: GenericTreeItem>(
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

impl<Item: GenericTreeItem> TreeData for Vec<Item> {
    type Identifier = Vec<Item::Identifier>;

    fn get_nodes(
        &self,
        open_identifiers: &HashSet<Self::Identifier>,
    ) -> Vec<Node<Self::Identifier>> {
        crate::flatten::flatten(open_identifiers, self, &[])
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

pub trait RecursiveSelect {
    type Identifier;

    fn child_direct<'root>(&'root self, identifier: &Self::Identifier) -> Option<&'root Self>;

    fn child_deep<'root>(&'root self, identifier: &[Self::Identifier]) -> Option<&'root Self> {
        let mut current = self;
        for identifier in identifier {
            current = current.child_direct(identifier)?;
        }
        Some(current)
    }
}

impl<Item: GenericTreeItem> RecursiveSelect for Item {
    type Identifier = Item::Identifier;

    fn child_direct<'root>(&'root self, identifier: &Self::Identifier) -> Option<&'root Self> {
        self.children()
            .iter()
            .find(|item| item.identifier() == identifier)
    }
}
