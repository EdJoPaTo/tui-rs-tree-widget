#![forbid(unsafe_code)]
#![warn(clippy::pedantic, clippy::nursery)]

/*!
Widget built to show Tree Data structures.

Tree widget [`Tree`] is generated with [`TreeItem`]s (which itself can contain [`TreeItem`] children to form the tree structure).
The user interaction state (like the current selection) is stored in the [`TreeState`].
*/

use std::collections::HashSet;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Block, Scrollbar, ScrollbarState, StatefulWidget, Widget};
use unicode_width::UnicodeWidthStr;

mod flatten;
mod tree_item;
mod tree_state;

pub use crate::flatten::Flattened;
pub use crate::tree_item::TreeItem;
pub use crate::tree_state::TreeState;

/// A `Tree` which can be rendered.
///
/// The generic argument `Identifier` is used to keep the state like the currently selected or opened [`TreeItem`]s in the [`TreeState`].
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
/// terminal.draw(|frame| {
///     let area = frame.size();
///
///     let tree_widget = Tree::new(items)
///         .expect("all item identifiers are unique")
///         .block(Block::bordered().title("Tree Widget"));
///
///     frame.render_stateful_widget(tree_widget, area, &mut state);
/// })?;
/// # Ok::<(), std::io::Error>(())
/// ```
#[derive(Debug, Clone)]
pub struct Tree<'a, Identifier> {
    items: Vec<TreeItem<'a, Identifier>>,

    block: Option<Block<'a>>,
    scrollbar: Option<Scrollbar<'a>>,
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
        let identifiers = items
            .iter()
            .map(|item| &item.identifier)
            .collect::<HashSet<_>>();
        if identifiers.len() != items.len() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                "The items contain duplicate identifiers",
            ));
        }

        Ok(Self {
            items,
            block: None,
            scrollbar: None,
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

    /// Show the scrollbar when rendering this widget.
    ///
    /// Experimental: Can change on any release without any additional notice.
    /// Its there to test and experiment with whats possible with scrolling widgets.
    /// Also see <https://github.com/ratatui-org/ratatui/issues/174>
    #[must_use]
    pub const fn experimental_scrollbar(mut self, scrollbar: Option<Scrollbar<'a>>) -> Self {
        self.scrollbar = scrollbar;
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
    let item = TreeItem::new_leaf("same", "text");
    let another = item.clone();
    Tree::new(vec![item, another]).unwrap();
}

impl<'a, Identifier> StatefulWidget for Tree<'a, Identifier>
where
    Identifier: Clone + PartialEq + Eq + core::hash::Hash,
{
    type State = TreeState<Identifier>;

    #[allow(clippy::too_many_lines)]
    fn render(self, full_area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        buf.set_style(full_area, self.style);

        // Get the inner area inside a possible block, otherwise use the full area
        let area = self.block.map_or(full_area, |block| {
            let inner_area = block.inner(full_area);
            block.render(full_area, buf);
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

        let ensure_index_in_view =
            if state.ensure_selected_in_view_on_next_render && !state.selected.is_empty() {
                visible
                    .iter()
                    .position(|flattened| flattened.identifier == state.selected)
            } else {
                None
            };

        // Ensure last line is still visible
        let mut start = state.offset.min(visible.len().saturating_sub(1));

        if let Some(ensure_index_in_view) = ensure_index_in_view {
            start = start.min(ensure_index_in_view);
        }

        let mut end = start;
        let mut height = 0;
        for item_height in visible
            .iter()
            .skip(start)
            .map(|flattened| flattened.item.height())
        {
            if height + item_height > available_height {
                break;
            }
            height += item_height;
            end += 1;
        }

        if let Some(ensure_index_in_view) = ensure_index_in_view {
            while ensure_index_in_view >= end {
                height += visible[end].item.height();
                end += 1;
                while height > available_height {
                    height = height.saturating_sub(visible[start].item.height());
                    start += 1;
                }
            }
        }

        state.offset = start;
        state.ensure_selected_in_view_on_next_render = false;

        if let Some(scrollbar) = self.scrollbar {
            let mut scrollbar_state = ScrollbarState::new(visible.len().saturating_sub(height))
                .position(start)
                .viewport_content_length(height);
            let scrollbar_area = Rect {
                // Inner height to be exactly as the content
                y: area.y,
                height: area.height,
                // Outer width to stay on the right border
                x: full_area.x,
                width: full_area.width,
            };
            scrollbar.render(scrollbar_area, buf, &mut scrollbar_state);
        }

        let blank_symbol = " ".repeat(self.highlight_symbol.width());

        let mut current_height = 0;
        let has_selection = !state.selected.is_empty();
        #[allow(clippy::cast_possible_truncation)]
        for flattened in visible.into_iter().skip(state.offset).take(end - start) {
            let Flattened {
                ref identifier,
                item,
            } = flattened;

            let x = area.x;
            let y = area.y + current_height;
            let height = item.height() as u16;
            current_height += height;

            let area = Rect {
                x,
                y,
                width: area.width,
                height,
            };

            let item_style = self.style.patch(item.style);
            buf.set_style(area, item_style);

            let is_selected = state.selected == *identifier;
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
                let indent_width = flattened.depth() * 2;
                let (after_indent_x, _) = buf.set_stringn(
                    after_highlight_symbol_x,
                    y,
                    " ".repeat(indent_width),
                    indent_width,
                    item_style,
                );
                let symbol = if item.children.is_empty() {
                    self.node_no_children_symbol
                } else if state.opened.contains(identifier) {
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
            for (j, line) in item.text.lines.iter().enumerate() {
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
