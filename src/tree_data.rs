use std::collections::HashSet;

use crate::Node;

pub trait TreeData
where
    Self: Sized,
{
    /// Unique identifier of the item in the current depth.
    ///
    /// The identifier is used to keep the state like the currently selected or opened nodes in the [`TreeState`](crate::TreeState).
    ///
    /// It needs to be unique among its siblings but can be used again on parent or child nodes.
    /// A common example would be a filename which has to be unique in its directory while it can exist in another.
    ///
    /// What is rendered can be different from its `identifier`.
    /// To repeat the filename analogy: File browsers sometimes hide file extensions.
    /// The filename `main.rs` is the identifier while its shown as `main`.
    /// Two files `main.rs` and `main.toml` can exist in the same directory and can both be displayed as `main` but their identifier is different.
    ///
    /// Just like every file in a file system can be uniquely identified with its file and directory names each node in a [`Tree`](crate::Tree) can be with these identifiers.
    /// As an example the following two identifiers describe the main file in a Rust cargo project: `vec!["src", "main.rs"]`.
    ///
    /// The identifier does not need to be a `String` and is therefore generic.
    /// Until version 0.14 this crate used `usize` and indices.
    /// This might still be perfect for your use case.
    type Identifier: Clone + PartialEq + Eq + core::hash::Hash;

    /// Returns all visible, accessable [`Node`]s in a flat `Vec`.
    ///
    /// The top level is always accessable while nodes need to be open for their children to be visible.
    /// Which are open/closed is stored in a [`TreeState`](crate::TreeState) which state is available here.
    fn get_nodes(
        &self,
        open_identifiers: &HashSet<Vec<Self::Identifier>>,
    ) -> Vec<Node<Self::Identifier>>;

    /// Render the given node to the buffer.
    ///
    /// Very similar to [`ratatui::widgets::Widget`].
    fn render(
        &self,
        identifier: &[Self::Identifier],
        area: ratatui::layout::Rect,
        buffer: &mut ratatui::buffer::Buffer,
    );

    fn total_required_height(&self, open_identifiers: &HashSet<Vec<Self::Identifier>>) -> usize {
        self.get_nodes(open_identifiers)
            .iter()
            .map(|node: &Node<<Self as TreeData>::Identifier>| node.height)
            .fold(0, usize::saturating_add)
    }
}
