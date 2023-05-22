#![allow(clippy::module_name_repetitions)]

/// Reference to a [`TreeItem`](crate::TreeItem) in a [`Tree`](crate::Tree)
pub type TreeIdentifier<'a> = &'a [usize];
/// Reference to a [`TreeItem`](crate::TreeItem) in a [`Tree`](crate::Tree)
pub type TreeIdentifierVec = Vec<usize>;

/// Split a [`TreeIdentifier`] into its branch and leaf
///
/// # Examples
///
/// ```
/// # use tui_tree_widget::get_identifier_without_leaf;
/// let (branch, leaf) = get_identifier_without_leaf(&[2, 4, 6]);
/// assert_eq!(branch, [2, 4]);
/// assert_eq!(leaf, Some(6));
///
/// let (branch, leaf) = get_identifier_without_leaf(&[2]);
/// assert_eq!(branch, []);
/// assert_eq!(leaf, Some(2));
///
/// let (branch, leaf) = get_identifier_without_leaf(&[]);
/// assert_eq!(branch, []);
/// assert_eq!(leaf, None);
/// ```
#[must_use]
pub const fn get_without_leaf(identifier: TreeIdentifier) -> (TreeIdentifier, Option<usize>) {
    match identifier {
        [branch @ .., leaf] => (branch, Some(*leaf)),
        [] => (&[] as &[usize], None),
    }
}
