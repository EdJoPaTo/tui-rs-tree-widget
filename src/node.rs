/// A node in a [`Tree`](crate::Tree).
///
/// Identified by an [Identifier](crate::TreeData::Identifier) and knows some information it will be rendered with.
#[must_use]
#[derive(Debug, PartialEq, Eq)]
pub struct Node<Identifier> {
    /// Zero based depth. Depth 0 means top level with 0 indentation.
    pub depth: usize,

    pub has_children: bool,

    pub height: usize,

    pub identifier: Vec<Identifier>,
}
