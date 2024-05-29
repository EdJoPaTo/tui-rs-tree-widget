use std::collections::HashSet;

use ratatui::layout::{Position, Rect};

use crate::Node;

/// Keeps the state of what is currently selected and what was opened in a [`Tree`](crate::Tree).
///
/// The generic argument [`Identifier`](crate::TreeData::Identifier) is used to keep the state like the currently selected or opened nodes in the [`TreeState`].
///
/// # Example
///
/// ```
/// # use tui_tree_widget::TreeState;
/// type Identifier = usize;
///
/// let mut state = TreeState::<Identifier>::default();
/// ```
#[must_use]
#[derive(Debug, Default)]
pub struct TreeState<Identifier> {
    pub(super) offset: usize,
    pub(super) opened: HashSet<Identifier>,
    pub(super) selected: Option<Identifier>,
    pub(super) ensure_selected_in_view_on_next_render: bool,

    pub(super) last_area: Rect,
    pub(super) last_biggest_index: usize,
    /// All nodes on last render
    pub(super) last_nodes: Vec<Node<Identifier>>,
    /// Identifier rendered at `y` on last render
    pub(super) last_rendered_identifiers: Vec<(u16, Identifier)>,
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
    pub const fn opened(&self) -> &HashSet<Identifier> {
        &self.opened
    }

    #[must_use]
    pub const fn selected(&self) -> Option<&Identifier> {
        self.selected.as_ref()
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
    pub fn select(&mut self, identifier: Option<Identifier>) -> bool {
        self.ensure_selected_in_view_on_next_render = true;
        let changed = self.selected != identifier;
        self.selected = identifier;
        changed
    }

    /// Open a tree node.
    /// Returns `true` when it was closed and has been opened.
    /// Returns `false` when it was already open.
    pub fn open(&mut self, identifier: Identifier) -> bool {
        self.opened.insert(identifier)
    }

    /// Close a tree node.
    /// Returns `true` when it was open and has been closed.
    /// Returns `false` when it was already closed.
    pub fn close(&mut self, identifier: &Identifier) -> bool {
        self.opened.remove(identifier)
    }

    /// Toggles a tree node open/close state.
    /// When it is currently open, then [`close`](Self::close) is called. Otherwise [`open`](Self::open).
    ///
    /// Returns `true` when a node is opened / closed.
    /// As toggle always changes something, this only returns `false` when an empty identifier is given.
    pub fn toggle(&mut self, identifier: Identifier) -> bool {
        if self.opened.contains(&identifier) {
            self.close(&identifier)
        } else {
            self.open(identifier)
        }
    }

    /// Toggles the currently selected tree node open/close state.
    /// See also [`toggle`](Self::toggle)
    ///
    /// Returns `true` when a node is opened / closed.
    /// As toggle always changes something, this only returns `false` when nothing is selected.
    pub fn toggle_selected(&mut self) -> bool {
        let Some(selected) = &self.selected else {
            return false;
        };

        self.ensure_selected_in_view_on_next_render = true;

        // Reimplement self.close because of multiple different borrows
        let was_open = self.opened.remove(selected);
        if was_open {
            return true;
        }

        self.open(selected.clone())
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
    pub fn select_first(&mut self) -> bool {
        let identifier = self.last_nodes.first().map(|node| node.identifier.clone());
        self.select(identifier)
    }

    /// Select the last node.
    ///
    /// Returns `true` when the selection changed.
    pub fn select_last(&mut self) -> bool {
        let new_identifier = self.last_nodes.last().map(|node| node.identifier.clone());
        self.select(new_identifier)
    }

    /// Select the node on the given index.
    ///
    /// Returns `true` when the selection changed.
    ///
    /// This can be useful for mouse clicks.
    #[deprecated = "Prefer self.click_at or self.rendered_at as visible index is hard to predict with height != 1"]
    pub fn select_visible_index(&mut self, new_index: usize) -> bool {
        let new_index = new_index.min(self.last_biggest_index);
        let new_identifier = self
            .last_nodes
            .get(new_index)
            .map(|node| node.identifier.clone());
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
    /// # type Identifier = usize;
    /// # let mut state = TreeState::<Identifier>::default();
    /// // Move the selection one down
    /// state.select_relative(|current| {
    ///     // When nothing is currently selected, select index 0
    ///     // Otherwise select current + 1 (without panicing)
    ///     current.map_or(0, |current| current.saturating_add(1))
    /// });
    /// ```
    ///
    /// For more examples take a look into the source code of [`key_up`](Self::key_up) or [`key_down`](Self::key_down).
    /// They are implemented with this method.
    pub fn select_relative<F>(&mut self, change_function: F) -> bool
    where
        F: FnOnce(Option<usize>) -> usize,
    {
        let current_index = self
            .last_nodes
            .iter()
            .position(|node| Some(&node.identifier) == self.selected.as_ref());
        let new_index = change_function(current_index).min(self.last_biggest_index);
        let new_identifier = self
            .last_nodes
            .get(new_index)
            .map(|node| node.identifier.clone());
        self.select(new_identifier)
    }

    /// Get the identifier that was rendered for the given position on last render.
    #[must_use]
    pub fn rendered_at(&self, position: Position) -> Option<&Identifier> {
        if !self.last_area.contains(position) {
            return None;
        }

        self.last_rendered_identifiers
            .iter()
            .rev()
            .find(|(y, _)| position.y >= *y)
            .map(|(_, identifier)| identifier)
    }

    /// Select what was rendered at the given position on last render.
    /// When it is already selected, toggle it.
    ///
    /// Returns `true` when the state changed.
    /// Returns `false` when there was nothing at the given position.
    pub fn click_at(&mut self, position: Position) -> bool {
        if let Some(identifier) = self.rendered_at(position) {
            if Some(identifier) == self.selected.as_ref() {
                self.toggle_selected()
            } else {
                self.select(Some(identifier.clone()))
            }
        } else {
            false
        }
    }

    /// Ensure the selected node is in view on next render
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
    /// Returns `true` when the scroll position changed.
    /// Returns `false` when the scrolling has reached the last node.
    pub fn scroll_down(&mut self, lines: usize) -> bool {
        let before = self.offset;
        self.offset = self
            .offset
            .saturating_add(lines)
            .min(self.last_biggest_index);
        before != self.offset
    }

    /// Handles the up arrow key.
    /// Moves up in the current depth or to its parent.
    ///
    /// Returns `true` when the selection changed.
    pub fn key_up(&mut self) -> bool {
        self.select_relative(|current| {
            // When nothing is selected, fall back to end
            current.map_or(usize::MAX, |current| current.saturating_sub(1))
        })
    }

    /// Handles the down arrow key.
    /// Moves down in the current depth or into a child node.
    ///
    /// Returns `true` when the selection changed.
    pub fn key_down(&mut self) -> bool {
        self.select_relative(|current| {
            // When nothing is selected, fall back to start
            current.map_or(0, |current| current.saturating_add(1))
        })
    }

    /// Handles the left arrow key.
    /// Closes the currently selected or moves to its parent.
    ///
    /// Returns `true` when the selection or the open state changed.
    pub fn key_left(&mut self) -> bool {
        let Some(selected) = &self.selected else {
            return false;
        };
        self.ensure_selected_in_view_on_next_render = true;

        // Reimplement self.close because of multiple different borrows
        let mut changed = self.opened.remove(selected);
        if !changed {
            let selected_depth = self
                .last_nodes
                .iter()
                .find(|node| node.identifier == *selected)
                .map(|node| node.depth)
                .filter(|depth| *depth > 0); // depth 0 doesnt have a parent
            if let Some(selected_depth) = selected_depth {
                let mut found_selection = false;
                let parent = self
                    .last_nodes
                    .iter()
                    .rev()
                    .skip_while(|node| {
                        found_selection = node.identifier == *selected;
                        !found_selection
                    })
                    .find(|node| node.depth < selected_depth)
                    .map(|node| node.identifier.clone());

                changed = self.select(parent);
            } else {
                changed = self.select(None);
            }
        }
        changed
    }

    /// Handles the right arrow key.
    /// Opens the currently selected.
    ///
    /// Returns `true` when it was closed and has been opened.
    /// Returns `false` when it was already open or nothing being selected.
    pub fn key_right(&mut self) -> bool {
        if let Some(selected) = &self.selected {
            self.ensure_selected_in_view_on_next_render = true;
            self.open(selected.clone())
        } else {
            false
        }
    }
}
