/// Split an `Identifier` into its branch and leaf.
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
/// let (branch, leaf) = get_identifier_without_leaf::<usize>(&[]);
/// assert_eq!(branch, []);
/// assert_eq!(leaf, None);
/// ```
#[must_use]
pub const fn get_without_leaf<Identifier>(
    identifier: &[Identifier],
) -> (&[Identifier], Option<&Identifier>) {
    match identifier {
        [branch @ .., leaf] => (branch, Some(leaf)),
        [] => (&[] as &[Identifier], None),
    }
}
