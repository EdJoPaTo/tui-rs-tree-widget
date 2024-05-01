use ratatui::text::Text;

use super::TreeItem;

/// One item inside a [`Tree`](crate::Tree).
///
/// For more information about the identifier see [`TreeItem::Identifier`].
///
/// # Performance
///
/// As this item accepts a [`Text`] it needs to be rendered with all the styles and so on before being created.
/// This means the computation cost of this implementation is on the item creation.
///
/// When there are many `TreeItem`s not all of them will end up being actually shown.
/// Some might be out of the view, some might be children not yet opened by the user.
/// Therefore, this implementation takes a lot of performance for pre-rendering items that might not even be visible in the current view.
///
/// It might be easier to use this `PrerenderedTreeItem` in contrast to specialized implementations of [`TreeItem`] (like `JsonTreeItem`) or implementing one yourself, but it will not be as efficient performance wise.
///
/// # Example
///
/// ```
/// # use tui_tree_widget::PrerenderedTreeItem;
/// let a = PrerenderedTreeItem::new_leaf("l", "Leaf");
/// let b = PrerenderedTreeItem::new("r", "Root", vec![a])?;
/// # Ok::<(), std::io::Error>(())
/// ```
#[derive(Debug, Clone)]
pub struct PrerenderedTreeItem<'text, Identifier> {
    identifier: Identifier,
    text: Text<'text>,
    children: Vec<Self>,
}

impl<'text, Identifier> PrerenderedTreeItem<'text, Identifier>
where
    Identifier: Clone + PartialEq + Eq + core::hash::Hash,
{
    /// Create a new `PrerenderedTreeItem` without children.
    #[must_use]
    pub fn new_leaf<T>(identifier: Identifier, text: T) -> Self
    where
        T: Into<Text<'text>>,
    {
        Self {
            identifier,
            text: text.into(),
            children: Vec::new(),
        }
    }

    /// Create a new `PrerenderedTreeItem` with children.
    ///
    /// # Errors
    ///
    /// Errors when there are duplicate identifiers in the children.
    #[track_caller]
    pub fn new<T>(identifier: Identifier, text: T, children: Vec<Self>) -> std::io::Result<Self>
    where
        T: Into<Text<'text>>,
    {
        crate::unique_identifiers::children(&children)?;
        Ok(Self {
            identifier,
            text: text.into(),
            children,
        })
    }

    /// Get a reference to a child by index.
    #[must_use]
    pub fn child(&self, index: usize) -> Option<&Self> {
        self.children.get(index)
    }

    /// Get a mutable reference to a child by index.
    #[must_use]
    pub fn child_mut(&mut self, index: usize) -> Option<&mut Self> {
        self.children.get_mut(index)
    }

    /// Add a child.
    ///
    /// # Errors
    ///
    /// Errors when the `identifier` of the `child` already exists in the children.
    #[track_caller]
    pub fn add_child(&mut self, child: Self) -> std::io::Result<()> {
        crate::unique_identifiers::add_child(&self.children, &child)?;
        self.children.push(child);
        Ok(())
    }
}

impl<Identifier> TreeItem for PrerenderedTreeItem<'_, Identifier>
where
    Identifier: Clone + PartialEq + Eq + core::hash::Hash,
{
    type Identifier = Identifier;

    fn children(&self) -> &[Self] {
        &self.children
    }

    fn height(&self) -> usize {
        self.text.height()
    }

    fn identifier(&self) -> &Self::Identifier {
        &self.identifier
    }

    fn render(&self, area: ratatui::layout::Rect, buffer: &mut ratatui::buffer::Buffer) {
        ratatui::widgets::Widget::render(&self.text, area, buffer);
    }
}
