use std::collections::HashSet;

use crate::{get_item, Node, TreeData};

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
