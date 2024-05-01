use ratatui::style::Style;

use super::TreeItem;

/// Represents a very simple [`TreeItem`] implementation.
///
/// It assumes the text to be shown is only a single line element and each text is different.
#[derive(Debug, Clone)]
pub struct SimpleTreeItem<'text> {
    text: &'text str,
    style: Style,
    children: Vec<SimpleTreeItem<'text>>,
}

impl<'text> SimpleTreeItem<'text> {
    /// Create a new `SimpleTreeItem` without children.
    #[must_use]
    pub const fn new_leaf(text: &'text str) -> Self {
        Self {
            text,
            style: Style::new(),
            children: Vec::new(),
        }
    }

    /// Create a new `SimpleTreeItem` with children.
    ///
    /// # Errors
    ///
    /// Errors when there are duplicate identifiers in the children.
    #[track_caller]
    pub fn new(text: &'text str, children: Vec<Self>) -> std::io::Result<Self> {
        crate::unique_identifiers::children(&children)?;
        Ok(Self {
            text,
            style: Style::new(),
            children,
        })
    }
}

impl<'text> TreeItem for SimpleTreeItem<'text> {
    type Identifier = &'text str;

    fn identifier(&self) -> &Self::Identifier {
        &self.text
    }

    fn children(&self) -> &[Self] {
        &self.children
    }

    fn height(&self) -> usize {
        1
    }

    fn render(&self, area: ratatui::layout::Rect, buffer: &mut ratatui::buffer::Buffer) {
        use ratatui::style::Styled;
        let text = self.text.set_style(self.style);
        ratatui::widgets::Widget::render(text, area, buffer);
    }
}

impl ratatui::style::Styled for SimpleTreeItem<'_> {
    type Item = Self;

    fn style(&self) -> Style {
        self.style
    }

    fn set_style<S: Into<Style>>(mut self, style: S) -> Self::Item {
        self.style = style.into();
        self
    }
}

impl SimpleTreeItem<'static> {
    #[cfg(test)]
    pub(crate) fn example() -> Vec<Self> {
        vec![
            Self::new_leaf("a"),
            Self::new(
                "b",
                vec![
                    Self::new_leaf("c"),
                    Self::new("d", vec![Self::new_leaf("e"), Self::new_leaf("f")])
                        .expect("all item identifiers are unique"),
                    Self::new_leaf("g"),
                ],
            )
            .expect("all item identifiers are unique"),
            Self::new_leaf("h"),
        ]
    }
}

#[test]
#[should_panic = "duplicate "]
fn tree_item_new_errors_with_duplicate_identifiers() {
    let item: SimpleTreeItem = SimpleTreeItem::new_leaf("same");
    let another = SimpleTreeItem::new_leaf("same");
    SimpleTreeItem::new("root", vec![item, another]).unwrap();
}
