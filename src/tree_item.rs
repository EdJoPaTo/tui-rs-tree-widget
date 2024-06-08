use std::collections::HashSet;

use ratatui::text::Text;

use crate::{GenericTreeItem, RecursiveSelectMut};

/// One item inside a [`Tree`](crate::Tree).
///
/// Can have zero or more `children`.
///
/// # Identifier
///
/// The generic argument `Identifier` is used to keep the state like the currently selected or opened [`TreeItem`]s in the [`TreeState`](crate::TreeState).
///
/// It needs to be unique among its siblings but can be used again on parent or child [`TreeItem`]s.
/// A common example would be a filename which has to be unique in its directory while it can exist in another.
///
/// The `text` can be different from its `identifier`.
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
///
/// # Example
///
/// ```
/// # use tui_tree_widget::TreeItem;
/// let a = TreeItem::new_leaf("l", "Leaf");
/// let b = TreeItem::new("r", "Root", vec![a])?;
/// # Ok::<(), std::io::Error>(())
/// ```
#[derive(Debug, Clone)]
pub struct TreeItem<'text, Identifier> {
    pub(super) identifier: Identifier,
    pub(super) text: Text<'text>,
    pub(super) children: Vec<Self>,
}

impl<'text, Identifier> TreeItem<'text, Identifier>
where
    Identifier: Clone + PartialEq + Eq + core::hash::Hash,
{
    /// Create a new `TreeItem` without children.
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

    /// Create a new `TreeItem` with children.
    ///
    /// # Errors
    ///
    /// Errors when there are duplicate identifiers in the children.
    pub fn new<T>(identifier: Identifier, text: T, children: Vec<Self>) -> std::io::Result<Self>
    where
        T: Into<Text<'text>>,
    {
        let identifiers = children
            .iter()
            .map(|item| &item.identifier)
            .collect::<HashSet<_>>();
        if identifiers.len() != children.len() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                "The children contain duplicate identifiers",
            ));
        }

        Ok(Self {
            identifier,
            text: text.into(),
            children,
        })
    }

    /// Get a reference to a child by index.
    #[must_use]
    #[deprecated = "use GenericTreeItem::child_direct"]
    pub fn child(&self, index: usize) -> Option<&Self> {
        self.children.get(index)
    }

    /// Get a mutable reference to a child by index.
    ///
    /// When you choose to change the `identifier` the [`TreeState`](crate::TreeState) might not work as expected afterwards.
    #[must_use]
    #[deprecated = "use RecursiveSelectMut::child_direct_mut"]
    pub fn child_mut(&mut self, index: usize) -> Option<&mut Self> {
        self.children.get_mut(index)
    }

    /// Add a child to the `TreeItem`.
    ///
    /// # Errors
    ///
    /// Errors when the `identifier` of the `child` already exists in the children.
    pub fn add_child(&mut self, child: Self) -> std::io::Result<()> {
        let existing = self
            .children
            .iter()
            .map(|item| &item.identifier)
            .collect::<HashSet<_>>();
        if existing.contains(&child.identifier) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                "identifier already exists in the children",
            ));
        }

        self.children.push(child);
        Ok(())
    }
}

impl TreeItem<'static, &'static str> {
    #[cfg(test)]
    #[must_use]
    pub(crate) fn example() -> Vec<Self> {
        vec![
            Self::new_leaf("a", "Alfa"),
            Self::new(
                "b",
                "Bravo",
                vec![
                    Self::new_leaf("c", "Charlie"),
                    Self::new(
                        "d",
                        "Delta",
                        vec![Self::new_leaf("e", "Echo"), Self::new_leaf("f", "Foxtrot")],
                    )
                    .expect("all item identifiers are unique"),
                    Self::new_leaf("g", "Golf"),
                ],
            )
            .expect("all item identifiers are unique"),
            Self::new_leaf("h", "Hotel"),
        ]
    }
}

#[test]
#[should_panic = "duplicate identifiers"]
fn tree_item_new_errors_with_duplicate_identifiers() {
    let item = TreeItem::new_leaf("same", "text");
    let another = item.clone();
    TreeItem::new("root", "Root", vec![item, another]).unwrap();
}

#[test]
#[should_panic = "identifier already exists"]
fn tree_item_add_child_errors_with_duplicate_identifiers() {
    let item = TreeItem::new_leaf("same", "text");
    let another = item.clone();
    let mut root = TreeItem::new("root", "Root", vec![item]).unwrap();
    root.add_child(another).unwrap();
}

impl<'text, Identifier> GenericTreeItem for TreeItem<'text, Identifier>
where
    Identifier: Clone + PartialEq + Eq + core::hash::Hash,
{
    type Identifier = Identifier;

    fn identifier(&self) -> &Self::Identifier {
        &self.identifier
    }

    fn children(&self) -> &[Self] {
        &self.children
    }

    fn height(&self) -> usize {
        self.text.height()
    }

    fn render(&self, area: ratatui::layout::Rect, buffer: &mut ratatui::buffer::Buffer) {
        ratatui::widgets::Widget::render(&self.text, area, buffer);
    }
}

impl<'text, Identifier> RecursiveSelectMut for TreeItem<'text, Identifier>
where
    Identifier: PartialEq,
{
    type Identifier = Identifier;

    fn child_direct_mut<'root>(
        &'root mut self,
        identifier: &Self::Identifier,
    ) -> Option<&'root mut Self> {
        self.children
            .iter_mut()
            .find(|item| &item.identifier == identifier)
    }
}
