use std::{error::Error, fmt};

use std::iter::zip;
use std::vec;

use tui::layout::{Alignment, Constraint};
use tui::style::Style;
use tui::text::StyledGrapheme;

use crate::reflow::{LineComposer, WordWrapper};
use crate::{Flattened, TreeItem};

#[derive(Debug)]
pub enum ComposeError {
    Constraint(Constraint),
}

impl Error for ComposeError {}

impl fmt::Display for ComposeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ComposeError::Constraint(constraint) => {
                write!(f, "Invalid constraint given: {:?}", constraint)
            }
        }
    }
}

enum ComposeMode {
    Fit(usize),
    Fill(usize, usize),
    Truncate(usize),
}

pub struct Composed<'a> {
    identifier: Vec<usize>,
    item: &'a TreeItem<'a>,
    text: Vec<Vec<StyledGrapheme<'a>>>,
}

impl<'a> Composed<'a> {
    #[must_use]
    pub fn depth(&self) -> usize {
        self.identifier.len() - 1
    }

    #[must_use]
    pub fn height(&self) -> usize {
        self.text.len()
    }

    #[must_use]
    pub fn identifier(&self) -> &Vec<usize> {
        &self.identifier
    }

    #[must_use]
    pub fn style(&self) -> Style {
        self.item.style
    }

    #[must_use]
    pub fn text(&self) -> &Vec<Vec<StyledGrapheme<'a>>> {
        &self.text
    }

    #[must_use]
    pub fn has_children(&self) -> bool {
        !self.item.children.is_empty()
    }
}

#[must_use]
pub fn compose<'a>(
    items: &'a [Flattened<'a>],
    width: u16,
    truncation_symbol: &'a str,
) -> Result<Vec<Composed<'a>>, ComposeError> {
    items
        .iter()
        .map(|flattened| {
            let mut composed = vec![];

            let heights = &flattened.item().heights;
            let paragraphs = &flattened.item().paragraphs;

            for (height, paragraph) in zip(heights, paragraphs) {
                let styled = paragraph.lines.iter().map(|line| {
                    (
                        line.0
                            .iter()
                            .flat_map(|span| span.styled_graphemes(flattened.item().style)),
                        Alignment::Left,
                    )
                });

                let mut composer = Box::new(WordWrapper::new(
                    styled,
                    width.saturating_sub(flattened.depth() as u16 * 2),
                    false,
                ));

                let mut lines = vec![];
                while let Some((line, _, _)) = composer.next_line() {
                    lines.push(line.to_vec());
                }

                let lines = match compose_mode(lines.len(), *height)? {
                    ComposeMode::Fit(take) => lines.into_iter().take(take).collect::<Vec<_>>(),
                    ComposeMode::Fill(take, fill) => {
                        let mut lines = lines.into_iter().take(take).collect::<Vec<_>>();
                        lines.extend((0..fill).into_iter().map(|_| vec![]).collect::<Vec<_>>());
                        lines
                    }
                    ComposeMode::Truncate(take) => {
                        let mut lines = lines.into_iter().take(take).collect::<Vec<_>>();
                        lines.push(vec![StyledGrapheme {
                            symbol: truncation_symbol,
                            style: flattened.item().style,
                        }]);
                        lines
                    }
                };

                composed.extend(lines);
            }

            Ok(Composed {
                identifier: flattened.identifier().clone(),
                item: flattened.item(),
                text: composed,
            })
        })
        .collect()
}

fn compose_mode(len: usize, constraint: Constraint) -> Result<ComposeMode, ComposeError> {
    let height = match constraint {
        Constraint::Length(height) => height,
        Constraint::Min(height) => std::cmp::max(len as u16, height),
        Constraint::Max(height) => std::cmp::min(len as u16, height),
        _ => return Err(ComposeError::Constraint(constraint).into()),
    } as usize;

    if len == height {
        Ok(ComposeMode::Fit(height))
    } else if len < height {
        Ok(ComposeMode::Fill(height, height.saturating_sub(len)))
    } else {
        Ok(ComposeMode::Truncate(height.saturating_sub(1)))
    }
}
