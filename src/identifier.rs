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
/// assert_eq!(leaf, Some(&6));
///
/// let (branch, leaf) = get_identifier_without_leaf(&[2]);
/// assert_eq!(branch, []);
/// assert_eq!(leaf, Some(&2));
///
/// let (branch, leaf) = get_identifier_without_leaf(&[]);
/// assert_eq!(branch, []);
/// assert_eq!(leaf, None);
/// ```
pub fn get_without_leaf(identifier: TreeIdentifier) -> (TreeIdentifier, Option<&usize>) {
    let length = identifier.len();
    let length_without_leaf = length.saturating_sub(1);

    let branch = &identifier[0..length_without_leaf];
    let leaf = identifier.get(length_without_leaf);

    (branch, leaf)
}
