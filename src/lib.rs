#![forbid(unsafe_code)]

/*!
Widget built to show Tree Data structures.

Tree widget [`Tree`] is generated with [`TreeItem`s](TreeItem) (which itself can contain [`TreeItem`] children to form the tree structure).
The user interaction state (like the current selection) is stored in the [`TreeState`].
*/

use std::collections::HashSet;

use ratatui::buffer::Buffer;
use ratatui::layout::{Corner, Rect};
use ratatui::style::Style;
use ratatui::text::Text;
use ratatui::widgets::{Block, StatefulWidget, Widget};
use unicode_width::UnicodeWidthStr;

mod flatten;
mod identifier;

use crate::flatten::flatten;
pub use crate::flatten::Flattened;
pub use crate::identifier::get_without_leaf as get_identifier_without_leaf;

/// Keeps the state of what is currently selected and what was opened in a [`Tree`].
///
/// The generic argument `Identifier` is used to keep the state like the currently selected or opened [`TreeItem`s](TreeItem) in the [`TreeState`].
/// For more information see [`TreeItem`].
///
/// # Example
///
/// ```
/// # use tui_tree_widget::TreeState;
/// type Identifier = usize;
///
/// let mut state = TreeState::<Identifier>::default();
/// ```
#[derive(Debug, Default, Clone)]
pub struct TreeState<Identifier> {
    offset: usize,
    opened: HashSet<Vec<Identifier>>,
    selected: Vec<Identifier>,
}

impl<Identifier> TreeState<Identifier>
where
    Identifier: Clone + PartialEq + Eq + core::hash::Hash,
{
    #[must_use]
    pub const fn get_offset(&self) -> usize {
        self.offset
    }

    #[must_use]
    pub fn get_all_opened(&self) -> Vec<Vec<Identifier>> {
        self.opened.iter().cloned().collect()
    }

    /// Get a flat list of all visible [`TreeItem`s](TreeItem) with this `TreeState`.
    #[must_use]
    pub fn flatten<'a>(
        &self,
        items: &'a [TreeItem<'a, Identifier>],
    ) -> Vec<Flattened<'a, Identifier>> {
        flatten(&self.opened, items)
    }

    #[must_use]
    pub fn selected(&self) -> Vec<Identifier> {
        self.selected.clone()
    }

    /// Selects the given identifier.
    ///
    /// Returns `true` when the selection changed.
    ///
    /// Clear the selection by passing an empty identifier vector:
    ///
    /// ```rust
    /// # use tui_tree_widget::TreeState;
    /// # let mut state = TreeState::<usize>::default();
    /// state.select(Vec::new());
    /// ```
    pub fn select(&mut self, identifier: Vec<Identifier>) -> bool {
        let changed = self.selected != identifier;
        self.selected = identifier;

        // TODO: ListState does this. Is this relevant?
        if self.selected.is_empty() {
            self.offset = 0;
        }

        changed
    }

    /// Open a tree node.
    /// Returns `true` if the node was closed and has been opened.
    /// Returns `false` if the node was already open.
    pub fn open(&mut self, identifier: Vec<Identifier>) -> bool {
        if identifier.is_empty() {
            false
        } else {
            self.opened.insert(identifier)
        }
    }

    /// Close a tree node.
    /// Returns `true` if the node was open and has been closed.
    /// Returns `false` if the node was already closed.
    pub fn close(&mut self, identifier: &[Identifier]) -> bool {
        self.opened.remove(identifier)
    }

    /// Toggles a tree node.
    /// If the node is in opened then it calls `close()`. Otherwise it calls `open()`.
    pub fn toggle(&mut self, identifier: Vec<Identifier>) {
        if self.opened.contains(&identifier) {
            self.close(&identifier);
        } else {
            self.open(identifier);
        }
    }

    /// Toggles the currently selected tree node.
    /// See also [`toggle`](TreeState::toggle)
    pub fn toggle_selected(&mut self) {
        self.toggle(self.selected());
    }

    pub fn close_all(&mut self) {
        self.opened.clear();
    }

    /// Select the first node.
    ///
    /// Returns `true` when the selection changed.
    pub fn select_first(&mut self, items: &[TreeItem<Identifier>]) -> bool {
        let identifier = items
            .first()
            .map(|o| vec![o.identifier.clone()])
            .unwrap_or_default();
        self.select(identifier)
    }

    /// Select the last visible node.
    ///
    /// Returns `true` when the selection changed.
    pub fn select_last(&mut self, items: &[TreeItem<Identifier>]) -> bool {
        let visible = self.flatten(items);
        let new_identifier = visible
            .last()
            .map(|o| o.identifier.clone())
            .unwrap_or_default();
        self.select(new_identifier)
    }

    /// Select the node visible on the given index.
    ///
    /// Returns `true` when the selection changed.
    ///
    /// This can be useful for mouse clicks.
    pub fn select_visible_index(
        &mut self,
        items: &[TreeItem<Identifier>],
        new_index: usize,
    ) -> bool {
        let visible = self.flatten(items);
        let new_index = new_index.min(visible.len().saturating_sub(1));
        let new_identifier = visible
            .get(new_index)
            .map(|o| o.identifier.clone())
            .unwrap_or_default();
        self.select(new_identifier)
    }

    /// Move the current selection with the direction/amount by the given function.
    ///
    /// Returns `true` when the selection changed.
    ///
    /// # Example
    ///
    /// ```
    /// # use tui_tree_widget::TreeState;
    /// # let items = vec![];
    /// # type Identifier = usize;
    /// # let mut state = TreeState::<Identifier>::default();
    /// // Move the selection one down
    /// state.select_visible_relative(&items, |current| {
    ///     current.map_or(0, |current| current.saturating_add(1))
    /// });
    /// ```
    ///
    /// For more examples take a look into the source code of [`TreeState::key_up`] or [`TreeState::key_down`].
    /// They are implemented with this method.
    pub fn select_visible_relative<F>(&mut self, items: &[TreeItem<Identifier>], f: F) -> bool
    where
        F: FnOnce(Option<usize>) -> usize,
    {
        let visible = self.flatten(items);
        let current_identifier = self.selected();
        let current_index = visible
            .iter()
            .position(|o| o.identifier == current_identifier);
        let new_index = f(current_index).min(visible.len().saturating_sub(1));
        let new_identifier = visible
            .get(new_index)
            .map(|o| o.identifier.clone())
            .unwrap_or_default();
        self.select(new_identifier)
    }

    /// Handles the up arrow key.
    /// Moves up in the current depth or to its parent.
    pub fn key_up(&mut self, items: &[TreeItem<Identifier>]) {
        self.select_visible_relative(items, |current| {
            current.map_or(usize::MAX, |current| current.saturating_sub(1))
        });
    }

    /// Handles the down arrow key.
    /// Moves down in the current depth or into a child node.
    pub fn key_down(&mut self, items: &[TreeItem<Identifier>]) {
        self.select_visible_relative(items, |current| {
            current.map_or(0, |current| current.saturating_add(1))
        });
    }

    /// Handles the left arrow key.
    /// Closes the currently selected or moves to its parent.
    pub fn key_left(&mut self) {
        // Reimplement self.close because of multiple different borrows
        let changed = self.opened.remove(&self.selected);
        if !changed {
            // Select the parent by removing the leaf from selection
            self.selected.pop();
        }
    }

    /// Handles the right arrow key.
    /// Opens the currently selected.
    pub fn key_right(&mut self) {
        self.open(self.selected());
    }
}

/// One item inside a [`Tree`].
///
/// Can have zero or more `children`.
///
/// # Identifier
///
/// The generic argument `Identifier` is used to keep the state like the currently selected or opened [`TreeItem`s](TreeItem) in the [`TreeState`].
///
/// It needs to be unique among its siblings but can be used again on parent or child [`TreeItem`s](TreeItem).
/// A common example would be a filename which has to be unique in its directory while it can exist in another.
///
/// The `text` can be different from its `identifier`.
/// To repeat the filename analogy: File browsers sometimes hide file extensions.
/// The filename `main.rs` is the identifier while its shown as `main`.
/// Two files `main.rs` and `main.toml` can exist in the same directory and can both be displayed as `main` but their identifier is different.
///
/// Just like every file in a file system can be uniquely identified with its file and directory names each [`TreeItem`] in a [`Tree`] can be with these identifiers.
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
pub struct TreeItem<'a, Identifier> {
    identifier: Identifier,
    text: Text<'a>,
    style: Style,
    children: Vec<TreeItem<'a, Identifier>>,
}

impl<'a, Identifier> TreeItem<'a, Identifier>
where
    Identifier: Clone + PartialEq + Eq + core::hash::Hash,
{
    /// Create a new `TreeItem` without children.
    #[must_use]
    pub fn new_leaf<T>(identifier: Identifier, text: T) -> Self
    where
        T: Into<Text<'a>>,
    {
        Self {
            identifier,
            text: text.into(),
            style: Style::new(),
            children: Vec::new(),
        }
    }

    /// Create a new `TreeItem` with children.
    ///
    /// # Errors
    ///
    /// Errors when there are duplicate identifiers in the children.
    pub fn new<T>(
        identifier: Identifier,
        text: T,
        children: Vec<TreeItem<'a, Identifier>>,
    ) -> std::io::Result<Self>
    where
        T: Into<Text<'a>>,
    {
        let identifiers = children
            .iter()
            .map(|o| &o.identifier)
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
            style: Style::new(),
            children,
        })
    }

    #[must_use]
    pub fn children(&self) -> &[TreeItem<Identifier>] {
        &self.children
    }

    /// Get a reference to a child by index.
    #[must_use]
    pub fn child(&self, index: usize) -> Option<&Self> {
        self.children.get(index)
    }

    /// Get a mutable reference to a child by index.
    ///
    /// When you choose to change the `identifier` the [`TreeState`] might not work as expected afterwards.
    #[must_use]
    pub fn child_mut(&mut self, index: usize) -> Option<&mut Self> {
        self.children.get_mut(index)
    }

    #[must_use]
    pub fn height(&self) -> usize {
        self.text.height()
    }

    #[must_use]
    pub const fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Add a child to the `TreeItem`.
    ///
    /// # Errors
    ///
    /// Errors when the `identifier` of the `child` already exists in the children.
    pub fn add_child(&mut self, child: TreeItem<'a, Identifier>) -> std::io::Result<()> {
        let existing = self
            .children
            .iter()
            .map(|o| &o.identifier)
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

#[test]
#[should_panic = "duplicate identifiers"]
fn tree_item_new_errors_with_duplicate_identifiers() {
    let a = TreeItem::new_leaf("same", "text");
    let b = a.clone();
    TreeItem::new("root", "Root", vec![a, b]).unwrap();
}

#[test]
#[should_panic = "identifier already exists"]
fn tree_item_add_child_errors_with_duplicate_identifiers() {
    let a = TreeItem::new_leaf("same", "text");
    let b = a.clone();
    let mut root = TreeItem::new("root", "Root", vec![a]).unwrap();
    root.add_child(b).unwrap();
}

/// A `Tree` which can be rendered.
///
/// The generic argument `Identifier` is used to keep the state like the currently selected or opened [`TreeItem`s](TreeItem) in the [`TreeState`].
/// For more information see [`TreeItem`].
///
/// # Example
///
/// ```
/// # use tui_tree_widget::{Tree, TreeItem, TreeState};
/// # use ratatui::backend::TestBackend;
/// # use ratatui::Terminal;
/// # use ratatui::widgets::Block;
/// # let mut terminal = Terminal::new(TestBackend::new(32, 32)).unwrap();
/// let mut state = TreeState::default();
///
/// let item = TreeItem::new_leaf("l", "leaf");
/// let items = vec![item];
///
/// terminal.draw(|f| {
///     let area = f.size();
///
///     let tree_widget = Tree::new(items)
///         .expect("all item identifiers are unique")
///         .block(Block::bordered().title("Tree Widget"));
///
///     f.render_stateful_widget(tree_widget, area, &mut state);
/// })?;
/// # Ok::<(), std::io::Error>(())
/// ```
#[derive(Debug, Clone)]
pub struct Tree<'a, Identifier> {
    items: Vec<TreeItem<'a, Identifier>>,

    block: Option<Block<'a>>,
    start_corner: Corner,
    /// Style used as a base style for the widget
    style: Style,

    /// Style used to render selected item
    highlight_style: Style,
    /// Symbol in front of the selected item (Shift all items to the right)
    highlight_symbol: &'a str,

    /// Symbol displayed in front of a closed node (As in the children are currently not visible)
    node_closed_symbol: &'a str,
    /// Symbol displayed in front of an open node. (As in the children are currently visible)
    node_open_symbol: &'a str,
    /// Symbol displayed in front of a node without children.
    node_no_children_symbol: &'a str,
}

impl<'a, Identifier> Tree<'a, Identifier>
where
    Identifier: Clone + PartialEq + Eq + core::hash::Hash,
{
    /// Create a new `Tree`.
    ///
    /// # Errors
    ///
    /// Errors when there are duplicate identifiers in the children.
    pub fn new(items: Vec<TreeItem<'a, Identifier>>) -> std::io::Result<Self> {
        let identifiers = items.iter().map(|o| &o.identifier).collect::<HashSet<_>>();
        if identifiers.len() != items.len() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                "The items contain duplicate identifiers",
            ));
        }

        Ok(Self {
            items,
            block: None,
            start_corner: Corner::TopLeft,
            style: Style::new(),
            highlight_style: Style::new(),
            highlight_symbol: "",
            node_closed_symbol: "\u{25b6} ", // Arrow to right
            node_open_symbol: "\u{25bc} ",   // Arrow down
            node_no_children_symbol: "  ",
        })
    }

    #[allow(clippy::missing_const_for_fn)]
    #[must_use]
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    #[must_use]
    pub const fn start_corner(mut self, corner: Corner) -> Self {
        self.start_corner = corner;
        self
    }

    #[must_use]
    pub const fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    #[must_use]
    pub const fn highlight_style(mut self, style: Style) -> Self {
        self.highlight_style = style;
        self
    }

    #[must_use]
    pub const fn highlight_symbol(mut self, highlight_symbol: &'a str) -> Self {
        self.highlight_symbol = highlight_symbol;
        self
    }

    #[must_use]
    pub const fn node_closed_symbol(mut self, symbol: &'a str) -> Self {
        self.node_closed_symbol = symbol;
        self
    }

    #[must_use]
    pub const fn node_open_symbol(mut self, symbol: &'a str) -> Self {
        self.node_open_symbol = symbol;
        self
    }

    #[must_use]
    pub const fn node_no_children_symbol(mut self, symbol: &'a str) -> Self {
        self.node_no_children_symbol = symbol;
        self
    }
}

#[test]
#[should_panic = "duplicate identifiers"]
fn tree_new_errors_with_duplicate_identifiers() {
    let a = TreeItem::new_leaf("same", "text");
    let b = a.clone();
    Tree::new(vec![a, b]).unwrap();
}

impl<'a, Identifier> StatefulWidget for Tree<'a, Identifier>
where
    Identifier: Clone + PartialEq + Eq + core::hash::Hash,
{
    type State = TreeState<Identifier>;

    #[allow(clippy::too_many_lines)]
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        buf.set_style(area, self.style);

        // Get the inner area inside a possible block, otherwise use the full area
        let area = self.block.map_or(area, |b| {
            let inner_area = b.inner(area);
            b.render(area, buf);
            inner_area
        });

        if area.width < 1 || area.height < 1 {
            return;
        }

        let visible = state.flatten(&self.items);
        if visible.is_empty() {
            return;
        }
        let available_height = area.height as usize;

        let selected_index = if state.selected.is_empty() {
            0
        } else {
            visible
                .iter()
                .position(|o| o.identifier == state.selected)
                .unwrap_or(0)
        };

        let mut start = state.offset.min(selected_index);
        let mut end = start;
        let mut height = 0;
        for item in visible.iter().skip(start) {
            if height + item.item.height() > available_height {
                break;
            }

            height += item.item.height();
            end += 1;
        }

        while selected_index >= end {
            height = height.saturating_add(visible[end].item.height());
            end += 1;
            while height > available_height {
                height = height.saturating_sub(visible[start].item.height());
                start += 1;
            }
        }

        state.offset = start;

        let blank_symbol = " ".repeat(self.highlight_symbol.width());

        let mut current_height = 0;
        let has_selection = !state.selected.is_empty();
        #[allow(clippy::cast_possible_truncation)]
        for item in visible.iter().skip(state.offset).take(end - start) {
            #[allow(clippy::single_match_else)] // Keep same as List impl
            let (x, y) = match self.start_corner {
                Corner::BottomLeft => {
                    current_height += item.item.height() as u16;
                    (area.left(), area.bottom() - current_height)
                }
                _ => {
                    let pos = (area.left(), area.top() + current_height);
                    current_height += item.item.height() as u16;
                    pos
                }
            };
            let area = Rect {
                x,
                y,
                width: area.width,
                height: item.item.height() as u16,
            };

            let item_style = self.style.patch(item.item.style);
            buf.set_style(area, item_style);

            let is_selected = state.selected == item.identifier;
            let after_highlight_symbol_x = if has_selection {
                let symbol = if is_selected {
                    self.highlight_symbol
                } else {
                    &blank_symbol
                };
                let (x, _) = buf.set_stringn(x, y, symbol, area.width as usize, item_style);
                x
            } else {
                x
            };

            let after_depth_x = {
                let indent_width = item.depth() * 2;
                let (after_indent_x, _) = buf.set_stringn(
                    after_highlight_symbol_x,
                    y,
                    " ".repeat(indent_width),
                    indent_width,
                    item_style,
                );
                let symbol = if item.item.children.is_empty() {
                    self.node_no_children_symbol
                } else if state.opened.contains(&item.identifier) {
                    self.node_open_symbol
                } else {
                    self.node_closed_symbol
                };
                let max_width = area.width.saturating_sub(after_indent_x - x);
                let (x, _) =
                    buf.set_stringn(after_indent_x, y, symbol, max_width as usize, item_style);
                x
            };

            let max_element_width = area.width.saturating_sub(after_depth_x - x);
            for (j, line) in item.item.text.lines.iter().enumerate() {
                buf.set_line(after_depth_x, y + j as u16, line, max_element_width);
            }
            if is_selected {
                buf.set_style(area, self.highlight_style);
            }
        }
    }
}

impl<'a, Identifier> Widget for Tree<'a, Identifier>
where
    Identifier: Clone + Default + Eq + core::hash::Hash,
{
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut state = TreeState::default();
        StatefulWidget::render(self, area, buf, &mut state);
    }
}
