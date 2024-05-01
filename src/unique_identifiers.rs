use std::collections::HashSet;
use std::io::ErrorKind;

use crate::TreeItem;

fn inner<Item>(items: &[Item], error: &'static str) -> std::io::Result<()>
where
    Item: TreeItem,
{
    let identifiers = items
        .iter()
        .map(TreeItem::identifier)
        .collect::<HashSet<_>>();
    if identifiers.len() == items.len() {
        Ok(())
    } else {
        Err(std::io::Error::new(ErrorKind::AlreadyExists, error))
    }
}

pub(super) fn tree<Item>(items: &[Item]) -> std::io::Result<()>
where
    Item: TreeItem,
{
    inner(items, "The items contain duplicate identifiers")
}

/// Ensures that all identifiers in the items are unique.
///
/// This is useful when you implement your own [`TreeItem`].
/// When you mark the calling function with `#[track_caller]` it gets easier to find the misbehaving caller.
///
/// # Errors
///
/// Errors when there are duplicate identifiers in the children.
#[track_caller]
pub fn children<Item>(children: &[Item]) -> std::io::Result<()>
where
    Item: TreeItem,
{
    inner(children, "The children contain duplicate identifiers")
}

/// Ensures that the to be added child identifier does not exist in the already existing children.
///
/// This is useful when you implement your own [`TreeItem`].
/// When you mark the calling function with `#[track_caller]` it gets easier to find the misbehaving caller.
///
/// # Errors
///
/// Errors when the new child would duplicate an existing identifier in the children.
#[track_caller]
pub fn add_child<Item>(existing_children: &[Item], add: &Item) -> std::io::Result<()>
where
    Item: TreeItem,
{
    let add_identifier = add.identifier();
    let identifier_exists_already = existing_children
        .iter()
        .map(Item::identifier)
        .any(|identifier| identifier == add_identifier);
    if identifier_exists_already {
        Err(std::io::Error::new(
            ErrorKind::AlreadyExists,
            "The to be added child identifier already exists in the children",
        ))
    } else {
        Ok(())
    }
}
