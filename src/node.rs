/// A node in a [`Tree`](crate::Tree).
///
/// Identified by an [Identifier](crate::TreeData::Identifier) and knows some information it will be rendered with.
#[must_use]
#[derive(Debug, PartialEq, Eq)]
pub struct Node<Identifier> {
    pub identifier: Vec<Identifier>,

    pub has_children: bool,
    pub height: usize,
}

impl<Identifier> Node<Identifier> {
    /// Zero based depth. Depth 0 means top level with 0 indentation.
    #[must_use]
    pub fn depth(&self) -> usize {
        self.identifier.len() - 1
    }
}
