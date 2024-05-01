/// One item inside a [`Tree`](crate::Tree).
///
/// Can have zero or more `children`.
pub trait TreeItem
where
    Self: Sized,
{
    /// Unique identifier of the item in the current depth.
    ///
    /// The identifier is used to keep the state like the currently selected or opened [`TreeItem`]s in the [`TreeState`](crate::TreeState).
    ///
    /// It needs to be unique among its siblings but can be used again on parent or child [`TreeItem`]s.
    /// A common example would be a filename which has to be unique in its directory while it can exist in another.
    ///
    /// What is rendered can be different from its `identifier`.
    /// To repeat the filename analogy: File browsers sometimes hide file extensions.
    /// The filename `main.rs` is the identifier while its shown as `main`.
    /// Two files `main.rs` and `main.toml` can exist in the same directory and can both be displayed as `main` but their identifier is different.
    ///
    /// Just like every file in a file system can be uniquely identified with its file and directory names each [`TreeItem`] in a [`Tree`](crate::Tree) can be with these identifiers.
    /// As an example the following two identifiers describe the main file in a Rust cargo project: `vec!["src", "main.rs"]`.
    ///
    /// The identifier does not need to be a `String` and is therefore generic.
    /// Until version 0.14 this crate used `usize` and indices.
    /// This might still be perfect for your use case.
    type Identifier: Clone + PartialEq + Eq + core::hash::Hash;

    /// Returns the direct children of this item.
    #[must_use]
    fn children(&self) -> &[Self];

    /// Returns the render height of this item.
    ///
    /// This is the height later available at [`render`](Self::render).
    /// It is used to check which items are visible when the full [`Tree`](crate::Tree) is being rendered.
    #[must_use]
    fn height(&self) -> usize;

    /// Returns a reference to the [`Identifier`](Self::Identifier) of the current item.
    #[must_use]
    fn identifier(&self) -> &Self::Identifier;

    /// Render this item to the buffer.
    ///
    /// Very similar to [`ratatui::widgets::Widget`] but only takes a reference of `self`.
    fn render(&self, area: ratatui::layout::Rect, buffer: &mut ratatui::buffer::Buffer);
}
