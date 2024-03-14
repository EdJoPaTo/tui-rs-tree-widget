use std::collections::HashSet;

use crate::flatten::{flatten, Flattened};
use crate::tree_item::TreeItem;

/// Keeps the state of what is currently selected and what was opened in a [`Tree`](crate::Tree).
///
/// The generic argument `Identifier` is used to keep the state like the currently selected or opened [`TreeItem`]s in the [`TreeState`].
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
    pub(super) offset: usize,
    pub(super) opened: HashSet<Vec<Identifier>>,
    pub(super) selected: Vec<Identifier>,
    pub(super) ensure_selected_in_view_on_next_render: bool,
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

    /// Get a flat list of all visible (= below open) [`TreeItem`]s with this `TreeState`.
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
        self.ensure_selected_in_view_on_next_render = true;
        let changed = self.selected != identifier;
        self.selected = identifier;
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
    /// If the node is in opened then it calls [`close`](Self::close). Otherwise it calls [`open`](Self::open).
    ///
    /// Returns `true` when a node is opened / closed.
    /// As toggle always changes something, this only returns `false` when an empty identifier is given.
    pub fn toggle(&mut self, identifier: Vec<Identifier>) -> bool {
        if identifier.is_empty() {
            false
        } else if self.opened.contains(&identifier) {
            self.close(&identifier)
        } else {
            self.open(identifier)
        }
    }

    /// Toggles the currently selected tree node.
    /// See also [`toggle`](Self::toggle)
    ///
    /// Returns `true` when a node is opened / closed.
    /// As toggle always changes something, this only returns `false` when nothing is selected.
    pub fn toggle_selected(&mut self) -> bool {
        self.ensure_selected_in_view_on_next_render = true;
        self.toggle(self.selected())
    }

    /// Closes all open nodes.
    ///
    /// Returns `true` when any node was closed.
    pub fn close_all(&mut self) -> bool {
        if self.opened.is_empty() {
            false
        } else {
            self.opened.clear();
            true
        }
    }

    /// Select the first node.
    ///
    /// Returns `true` when the selection changed.
    pub fn select_first(&mut self, items: &[TreeItem<Identifier>]) -> bool {
        let identifier = items
            .first()
            .map_or(Vec::new(), |item| vec![item.identifier.clone()]);
        self.select(identifier)
    }

    /// Select the last visible node.
    ///
    /// Returns `true` when the selection changed.
    pub fn select_last(&mut self, items: &[TreeItem<Identifier>]) -> bool {
        let visible = self.flatten(items);
        let new_identifier = visible
            .into_iter()
            .last()
            .map_or(Vec::new(), |flattened| flattened.identifier);
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
            .into_iter()
            .nth(new_index)
            .map_or(Vec::new(), |flattened| flattened.identifier);
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
    /// For more examples take a look into the source code of [`key_up`](Self::key_up) or [`key_down`](Self::key_down).
    /// They are implemented with this method.
    pub fn select_visible_relative<F>(
        &mut self,
        items: &[TreeItem<Identifier>],
        change_function: F,
    ) -> bool
    where
        F: FnOnce(Option<usize>) -> usize,
    {
        let visible = self.flatten(items);
        let current_identifier = self.selected();
        let current_index = visible
            .iter()
            .position(|flattened| flattened.identifier == current_identifier);
        let new_index = change_function(current_index).min(visible.len().saturating_sub(1));
        let new_identifier = visible
            .into_iter()
            .nth(new_index)
            .map_or(Vec::new(), |flattened| flattened.identifier);
        self.select(new_identifier)
    }

    /// Ensure the selected [`TreeItem`] is visible on next render
    pub fn scroll_selected_into_view(&mut self) {
        self.ensure_selected_in_view_on_next_render = true;
    }

    /// Scroll the specified amount of lines up
    ///
    /// Returns `true` when the scroll position changed.
    /// Returns `false` when the scrolling has reached the top.
    pub fn scroll_up(&mut self, lines: usize) -> bool {
        let before = self.offset;
        self.offset = self.offset.saturating_sub(lines);
        before != self.offset
    }

    /// Scroll the specified amount of lines down
    ///
    /// In contrast to [`scroll_up()`](Self::scroll_up) this can not return whether the view position changed or not as the actual change is determined on render.
    /// Always returns `true`.
    pub fn scroll_down(&mut self, lines: usize) -> bool {
        self.offset = self.offset.saturating_add(lines);
        true
    }

    /// Handles the up arrow key.
    /// Moves up in the current depth or to its parent.
    ///
    /// Returns `true` when the selection changed.
    pub fn key_up(&mut self, items: &[TreeItem<Identifier>]) -> bool {
        self.select_visible_relative(items, |current| {
            current.map_or(usize::MAX, |current| current.saturating_sub(1))
        })
    }

    /// Handles the down arrow key.
    /// Moves down in the current depth or into a child node.
    ///
    /// Returns `true` when the selection changed.
    pub fn key_down(&mut self, items: &[TreeItem<Identifier>]) -> bool {
        self.select_visible_relative(items, |current| {
            current.map_or(0, |current| current.saturating_add(1))
        })
    }

    /// Handles the left arrow key.
    /// Closes the currently selected or moves to its parent.
    ///
    /// Returns `true` when the selection or the open state changed.
    pub fn key_left(&mut self) -> bool {
        self.ensure_selected_in_view_on_next_render = true;
        // Reimplement self.close because of multiple different borrows
        let mut changed = self.opened.remove(&self.selected);
        if !changed {
            // Select the parent by removing the leaf from selection
            let popped = self.selected.pop();
            changed = popped.is_some();
        }
        changed
    }

    /// Handles the right arrow key.
    /// Opens the currently selected.
    ///
    /// Returns `true` if the node was closed and has been opened.
    /// Returns `false` if the node was already open.
    pub fn key_right(&mut self) -> bool {
        self.ensure_selected_in_view_on_next_render = true;
        self.open(self.selected())
    }
}
