//! Widget built to show Tree Data structures.
//!
//! Tree widget [`Tree`] is generated with [`TreeItem`]s (which itself can contain [`TreeItem`] children to form the tree structure).
//! The user interaction state (like the current selection) is stored in the [`TreeState`].

use std::collections::HashSet;

use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::style::Style;
use ratatui_core::widgets::{StatefulWidget, Widget};
pub use ratatui_widgets::block::Block;
pub use ratatui_widgets::scrollbar::{Scrollbar, ScrollbarState};
use unicode_width::UnicodeWidthStr as _;

pub use crate::flatten::Flattened;
pub use crate::tree_item::TreeItem;
pub use crate::tree_state::TreeState;

mod flatten;
mod tree_item;
mod tree_state;

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
///     let area = frame.area();
///
///     let tree_widget = Tree::new(&items)
///         .expect("all item identifiers are unique")
///         .block(Block::bordered().title("Tree Widget"));
///
///     frame.render_stateful_widget(tree_widget, area, &mut state);
/// })?;
/// # Ok::<(), std::convert::Infallible>(())
/// ```
#[must_use]
#[derive(Debug, Clone)]
pub struct Tree<'a, Identifier> {
    items: &'a [TreeItem<'a, Identifier>],

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

    /// When enabled, draw indent guide lines (tree branch connectors) in place
    /// of the blank indentation. Disabled by default for backward compatibility.
    indent_guides: bool,
    /// Style applied to the indent guide symbols.
    indent_guide_style: Style,
    /// Guide drawn for an ancestor level that continues below (`│ `).
    indent_guide_vertical_symbol: &'a str,
    /// Connector drawn in front of an item that has following siblings (`├─`).
    indent_guide_branch_symbol: &'a str,
    /// Connector drawn in front of the last item at its level (`└─`).
    indent_guide_last_branch_symbol: &'a str,
    /// Padding drawn for an ancestor level that has ended (no vertical line).
    indent_guide_padding_symbol: &'a str,
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
    pub fn new(items: &'a [TreeItem<'a, Identifier>]) -> std::io::Result<Self> {
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
            indent_guides: false,
            indent_guide_style: Style::new(),
            indent_guide_vertical_symbol: "\u{2502} ", // "│ "
            indent_guide_branch_symbol: "\u{251c}\u{2500}", // "├─"
            indent_guide_last_branch_symbol: "\u{2514}\u{2500}", // "└─"
            indent_guide_padding_symbol: "  ",
        })
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    /// Show the scrollbar when rendering this widget.
    ///
    /// Experimental: Can change on any release without any additional notice.
    /// Its there to test and experiment with whats possible with scrolling widgets.
    /// Also see <https://github.com/ratatui-org/ratatui/issues/174>
    pub const fn experimental_scrollbar(mut self, scrollbar: Option<Scrollbar<'a>>) -> Self {
        self.scrollbar = scrollbar;
        self
    }

    pub const fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub const fn highlight_style(mut self, style: Style) -> Self {
        self.highlight_style = style;
        self
    }

    pub const fn highlight_symbol(mut self, highlight_symbol: &'a str) -> Self {
        self.highlight_symbol = highlight_symbol;
        self
    }

    pub const fn node_closed_symbol(mut self, symbol: &'a str) -> Self {
        self.node_closed_symbol = symbol;
        self
    }

    pub const fn node_open_symbol(mut self, symbol: &'a str) -> Self {
        self.node_open_symbol = symbol;
        self
    }

    pub const fn node_no_children_symbol(mut self, symbol: &'a str) -> Self {
        self.node_no_children_symbol = symbol;
        self
    }

    /// Enable or disable indent guide lines.
    ///
    /// When enabled, the blank indentation in front of nested items is replaced
    /// with tree branch connectors (`│`, `├─`, `└─`). Disabled by default.
    ///
    /// Customize the individual glyphs with [`indent_guide_vertical_symbol`](Self::indent_guide_vertical_symbol),
    /// [`indent_guide_branch_symbol`](Self::indent_guide_branch_symbol),
    /// [`indent_guide_last_branch_symbol`](Self::indent_guide_last_branch_symbol) and
    /// [`indent_guide_padding_symbol`](Self::indent_guide_padding_symbol),
    /// and their appearance with [`indent_guide_style`](Self::indent_guide_style).
    pub const fn indent_guides(mut self, enabled: bool) -> Self {
        self.indent_guides = enabled;
        self
    }

    /// Style applied to the indent guide symbols.
    pub const fn indent_guide_style(mut self, style: Style) -> Self {
        self.indent_guide_style = style;
        self
    }

    /// Guide drawn for an ancestor level that continues below (default `"│ "`).
    ///
    /// Should be two columns wide to keep items aligned across depths.
    pub const fn indent_guide_vertical_symbol(mut self, symbol: &'a str) -> Self {
        self.indent_guide_vertical_symbol = symbol;
        self
    }

    /// Connector drawn in front of an item that has following siblings (default `"├─"`).
    ///
    /// Should be two columns wide to keep items aligned across depths.
    pub const fn indent_guide_branch_symbol(mut self, symbol: &'a str) -> Self {
        self.indent_guide_branch_symbol = symbol;
        self
    }

    /// Connector drawn in front of the last item at its level (default `"└─"`).
    ///
    /// Should be two columns wide to keep items aligned across depths.
    pub const fn indent_guide_last_branch_symbol(mut self, symbol: &'a str) -> Self {
        self.indent_guide_last_branch_symbol = symbol;
        self
    }

    /// Padding drawn for an ancestor level that has ended (default `"  "`).
    ///
    /// Should be two columns wide to keep items aligned across depths.
    pub const fn indent_guide_padding_symbol(mut self, symbol: &'a str) -> Self {
        self.indent_guide_padding_symbol = symbol;
        self
    }
}

#[test]
#[should_panic = "duplicate identifiers"]
fn tree_new_errors_with_duplicate_identifiers() {
    let item = TreeItem::new_leaf("same", "text");
    let another = item.clone();
    let items = [item, another];
    let _: Tree<_> = Tree::new(&items).unwrap();
}

impl<Identifier> StatefulWidget for Tree<'_, Identifier>
where
    Identifier: Clone + PartialEq + Eq + core::hash::Hash,
{
    type State = TreeState<Identifier>;

    #[expect(clippy::too_many_lines)]
    fn render(self, full_area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        buf.set_style(full_area, self.style);

        // Get the inner area inside a possible block, otherwise use the full area
        let area = self.block.map_or(full_area, |block| {
            let inner_area = block.inner(full_area);
            block.render(full_area, buf);
            inner_area
        });

        state.last_area = area;
        state.last_rendered_identifiers.clear();
        if area.width < 1 || area.height < 1 {
            return;
        }

        let visible = state.flatten(self.items);
        state.last_biggest_index = visible.len().saturating_sub(1);
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
        let mut start = state.offset.min(state.last_biggest_index);

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
        #[expect(clippy::cast_possible_truncation)]
        for flattened in visible.iter().skip(state.offset).take(end - start) {
            let Flattened {
                identifier, item, ..
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

            let text = &item.text;
            let item_style = text.style;

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
                let depth = flattened.depth();
                let (indent, indent_style) = if self.indent_guides && depth > 0 {
                    let mut guides = String::new();
                    for level in 1..=depth {
                        let has_next_sibling = flattened.has_next_sibling[level];
                        let guide = if level == depth {
                            if has_next_sibling {
                                self.indent_guide_branch_symbol
                            } else {
                                self.indent_guide_last_branch_symbol
                            }
                        } else if has_next_sibling {
                            self.indent_guide_vertical_symbol
                        } else {
                            self.indent_guide_padding_symbol
                        };
                        guides.push_str(guide);
                    }
                    (guides, item_style.patch(self.indent_guide_style))
                } else {
                    (" ".repeat(depth * 2), item_style)
                };
                let (after_indent_x, _) = buf.set_stringn(
                    after_highlight_symbol_x,
                    y,
                    &indent,
                    indent.width(),
                    indent_style,
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

            let text_area = Rect {
                x: after_depth_x,
                width: area.width.saturating_sub(after_depth_x - x),
                ..area
            };
            text.render(text_area, buf);

            if is_selected {
                buf.set_style(area, self.highlight_style);
            }

            state
                .last_rendered_identifiers
                .push((area.y, identifier.clone()));
        }
        state.last_identifiers = visible
            .into_iter()
            .map(|flattened| flattened.identifier)
            .collect();
    }
}

impl<Identifier> Widget for Tree<'_, Identifier>
where
    Identifier: Clone + Eq + core::hash::Hash,
{
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut state = TreeState::default();
        StatefulWidget::render(self, area, buf, &mut state);
    }
}

#[cfg(test)]
mod render_tests {
    use super::*;

    #[must_use]
    #[track_caller]
    fn render(width: u16, height: u16, state: &mut TreeState<&'static str>) -> Buffer {
        let items = TreeItem::example();
        let tree = Tree::new(&items).unwrap();
        let area = Rect::new(0, 0, width, height);
        let mut buffer = Buffer::empty(area);
        StatefulWidget::render(tree, area, &mut buffer, state);
        buffer
    }

    #[test]
    fn does_not_panic() {
        _ = render(0, 0, &mut TreeState::default());
        _ = render(10, 0, &mut TreeState::default());
        _ = render(0, 10, &mut TreeState::default());
        _ = render(10, 10, &mut TreeState::default());
    }

    #[test]
    fn nothing_open() {
        let buffer = render(10, 4, &mut TreeState::default());
        #[rustfmt::skip]
        let expected = Buffer::with_lines([
            "  Alfa    ",
            "▶ Bravo   ",
            "  Hotel   ",
            "          ",
        ]);
        assert_eq!(buffer, expected);
    }

    #[test]
    fn depth_one() {
        let mut state = TreeState::default();
        state.open(vec!["b"]);
        let buffer = render(13, 7, &mut state);
        let expected = Buffer::with_lines([
            "  Alfa       ",
            "▼ Bravo      ",
            "    Charlie  ",
            "  ▶ Delta    ",
            "    Golf     ",
            "  Hotel      ",
            "             ",
        ]);
        assert_eq!(buffer, expected);
    }

    #[test]
    fn depth_two() {
        let mut state = TreeState::default();
        state.open(vec!["b"]);
        state.open(vec!["b", "d"]);
        let buffer = render(15, 9, &mut state);
        let expected = Buffer::with_lines([
            "  Alfa         ",
            "▼ Bravo        ",
            "    Charlie    ",
            "  ▼ Delta      ",
            "      Echo     ",
            "      Foxtrot  ",
            "    Golf       ",
            "  Hotel        ",
            "               ",
        ]);
        assert_eq!(buffer, expected);
    }

    #[must_use]
    #[track_caller]
    fn render_with_guides(width: u16, height: u16, state: &mut TreeState<&'static str>) -> Buffer {
        let items = TreeItem::example();
        let tree = Tree::new(&items).unwrap().indent_guides(true);
        let area = Rect::new(0, 0, width, height);
        let mut buffer = Buffer::empty(area);
        StatefulWidget::render(tree, area, &mut buffer, state);
        buffer
    }

    #[test]
    fn indent_guides_do_not_panic_when_narrow() {
        let mut state = TreeState::default();
        state.open(vec!["b"]);
        state.open(vec!["b", "d"]);
        // Widths narrower than the guide indentation must clamp, not panic.
        for width in 0..8 {
            _ = render_with_guides(width, 9, &mut state);
        }
    }

    #[test]
    fn indent_guides_depth_two() {
        let mut state = TreeState::default();
        state.open(vec!["b"]);
        state.open(vec!["b", "d"]);
        let buffer = render_with_guides(18, 9, &mut state);
        let expected = Buffer::with_lines([
            "  Alfa            ",
            "▼ Bravo           ",
            "├─  Charlie       ",
            "├─▼ Delta         ",
            "│ ├─  Echo        ",
            "│ └─  Foxtrot     ",
            "└─  Golf          ",
            "  Hotel           ",
            "                  ",
        ]);
        assert_eq!(buffer, expected);
    }

    #[test]
    fn indent_guides_custom_symbols() {
        let mut state = TreeState::default();
        state.open(vec!["b"]);
        let items = TreeItem::example();
        let tree = Tree::new(&items)
            .unwrap()
            .indent_guides(true)
            .indent_guide_vertical_symbol(".:")
            .indent_guide_branch_symbol("+-")
            .indent_guide_last_branch_symbol("\\-")
            .indent_guide_padding_symbol("__");
        let area = Rect::new(0, 0, 13, 7);
        let mut buffer = Buffer::empty(area);
        StatefulWidget::render(tree, area, &mut buffer, &mut state);
        let expected = Buffer::with_lines([
            "  Alfa       ",
            "▼ Bravo      ",
            "+-  Charlie  ",
            "+-▶ Delta    ",
            "\\-  Golf     ",
            "  Hotel      ",
            "             ",
        ]);
        assert_eq!(buffer, expected);
    }

    #[test]
    fn indent_guides_style_and_selection() {
        use ratatui_core::style::Color;

        let mut state = TreeState::default();
        state.open(vec!["b"]);
        state.open(vec!["b", "d"]);
        state.select(vec!["b", "d"]); // Delta, a nested row at depth 1

        let items = TreeItem::example();
        let tree = Tree::new(&items)
            .unwrap()
            .indent_guides(true)
            .indent_guide_style(Style::new().fg(Color::Red))
            .highlight_style(Style::new().fg(Color::Blue));
        let area = Rect::new(0, 0, 18, 9);
        let mut buffer = Buffer::empty(area);
        StatefulWidget::render(tree, area, &mut buffer, &mut state);

        // Guides are drawn regardless of selection; the highlight symbol is the
        // default empty string, so layout matches `indent_guides_depth_two`.
        // y=2 is "├─  Charlie" (not selected), y=3 is "├─▼ Delta" (selected).
        let charlie_guide = buffer.cell((0_u16, 2_u16)).unwrap();
        assert_eq!(charlie_guide.symbol(), "\u{251c}"); // '├'
        // On a non-selected row the guide keeps `indent_guide_style`.
        assert_eq!(charlie_guide.fg, Color::Red);

        let delta_guide = buffer.cell((0_u16, 3_u16)).unwrap();
        assert_eq!(delta_guide.symbol(), "\u{251c}"); // '├'
        // On the selected row the full-row highlight repaints over the guide,
        // so `highlight_style` wins. This is the expected selection behavior:
        // the highlight bar spans the whole line, guides included.
        assert_eq!(delta_guide.fg, Color::Blue);
    }
}
