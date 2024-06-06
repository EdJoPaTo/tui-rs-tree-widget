use std::collections::HashSet;

use crate::{Node, TreeData};

pub trait GenericTreeItem
where
    Self: Sized,
{
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
fn get_item_direct<'root, Item: GenericTreeItem>(
    root: &'root [Item],
    identifier: &Item::Identifier,
) -> Option<&'root Item> {
    root.iter().find(|item| item.identifier() == identifier)
}

#[must_use]
fn get_item<'root, Item: GenericTreeItem>(
    root: &'root [Item],
    identifier: &[Item::Identifier],
) -> Option<&'root Item> {
    let mut identifier = identifier.iter();
    let mut current = get_item_direct(root, identifier.next()?)?;
    for identifier in identifier {
        current = get_item_direct(current.children(), identifier)?;
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
