use std::{collections::VecDeque, vec::IntoIter};

use unicode_width::UnicodeWidthStr;

use tui::{layout::Alignment, text::StyledGrapheme};

const NBSP: &str = "\u{00a0}";

/// A state machine to pack styled symbols into lines.
/// Cannot implement it as Iterator since it yields slices of the internal buffer (need streaming
/// iterators for that).
pub trait LineComposer<'a> {
    fn next_line(&mut self) -> Option<(&[StyledGrapheme<'a>], u16, Alignment)>;
}

/// A state machine that wraps lines on word boundaries.
pub struct WordWrapper<'a, O, I>
where
    // Outer iterator providing the individual lines
    O: Iterator<Item = (I, Alignment)>,
    // Inner iterator providing the styled symbols of a line Each line consists of an alignment and
    // a series of symbols
    I: Iterator<Item = StyledGrapheme<'a>>,
{
    /// The given, unprocessed lines
    input_lines: O,
    max_line_width: u16,
    wrapped_lines: Option<IntoIter<Vec<StyledGrapheme<'a>>>>,
    current_alignment: Alignment,
    current_line: Vec<StyledGrapheme<'a>>,
    /// Removes the leading whitespace from lines
    trim: bool,
}

impl<'a, O, I> WordWrapper<'a, O, I>
where
    O: Iterator<Item = (I, Alignment)>,
    I: Iterator<Item = StyledGrapheme<'a>>,
{
    pub fn new(lines: O, max_line_width: u16, trim: bool) -> WordWrapper<'a, O, I> {
        WordWrapper {
            input_lines: lines,
            max_line_width,
            wrapped_lines: None,
            current_alignment: Alignment::Left,
            current_line: vec![],
            trim,
        }
    }
}

impl<'a, O, I> LineComposer<'a> for WordWrapper<'a, O, I>
where
    O: Iterator<Item = (I, Alignment)>,
    I: Iterator<Item = StyledGrapheme<'a>>,
{
    fn next_line(&mut self) -> Option<(&[StyledGrapheme<'a>], u16, Alignment)> {
        if self.max_line_width == 0 {
            return None;
        }

        let mut current_line: Option<Vec<StyledGrapheme<'a>>> = None;
        let mut line_width: u16 = 0;

        // Try to repeatedly retrieve next line
        while current_line.is_none() {
            // Retrieve next preprocessed wrapped line
            if let Some(line_iterator) = &mut self.wrapped_lines {
                if let Some(line) = line_iterator.next() {
                    line_width = line
                        .iter()
                        .map(|grapheme| grapheme.symbol.width())
                        .sum::<usize>() as u16;
                    current_line = Some(line);
                }
            }

            // When no more preprocessed wrapped lines
            if current_line.is_none() {
                // Try to calculate next wrapped lines based on current whole line
                if let Some((line_symbols, line_alignment)) = &mut self.input_lines.next() {
                    // Save the whole line's alignment
                    self.current_alignment = *line_alignment;
                    let mut wrapped_lines = vec![]; // Saves the wrapped lines
                                                    // Saves the unfinished wrapped line
                    let (mut current_line, mut current_line_width) = (vec![], 0);
                    // Saves the partially processed word
                    let (mut unfinished_word, mut word_width) = (vec![], 0);
                    // Saves the whitespaces of the partially unfinished word
                    let (mut unfinished_whitespaces, mut whitespace_width) =
                        (VecDeque::<StyledGrapheme>::new(), 0);

                    let mut has_seen_non_whitespace = false;
                    for StyledGrapheme { symbol, style } in line_symbols {
                        let symbol_whitespace =
                            symbol.chars().all(&char::is_whitespace) && symbol != NBSP;
                        let symbol_width = symbol.width() as u16;
                        // Ignore characters wider than the total max width
                        if symbol_width > self.max_line_width {
                            continue;
                        }

                        // Append finished word to current line
                        if has_seen_non_whitespace && symbol_whitespace
                            // Append if trimmed (whitespaces removed) word would overflow
                            || word_width + symbol_width > self.max_line_width && current_line.is_empty() && self.trim
                            // Append if removed whitespace would overflow -> reset whitespace counting to prevent overflow
                            || whitespace_width + symbol_width > self.max_line_width && current_line.is_empty() && self.trim
                            // Append if complete word would overflow
                            || word_width + whitespace_width + symbol_width > self.max_line_width && current_line.is_empty() && !self.trim
                        {
                            if !current_line.is_empty() || !self.trim {
                                // Also append whitespaces if not trimming or current line is not
                                // empty
                                current_line.extend(
                                    std::mem::take(&mut unfinished_whitespaces).into_iter(),
                                );
                                current_line_width += whitespace_width;
                            }
                            // Append trimmed word
                            current_line.append(&mut unfinished_word);
                            current_line_width += word_width;

                            // Clear whitespace buffer
                            unfinished_whitespaces.clear();
                            whitespace_width = 0;
                            word_width = 0;
                        }

                        // Append the unfinished wrapped line to wrapped lines if it is as wide as
                        // max line width
                        if current_line_width >= self.max_line_width
                            // or if it would be too long with the current partially processed word added
                            || current_line_width + whitespace_width + word_width >= self.max_line_width && symbol_width > 0
                        {
                            let mut remaining_width =
                                (self.max_line_width as i32 - current_line_width as i32).max(0)
                                    as u16;
                            wrapped_lines.push(std::mem::take(&mut current_line));
                            current_line_width = 0;

                            // Remove all whitespaces till end of just appended wrapped line + next
                            // whitespace
                            let mut first_whitespace = unfinished_whitespaces.pop_front();
                            while let Some(grapheme) = first_whitespace.as_ref() {
                                let symbol_width = grapheme.symbol.width() as u16;
                                whitespace_width -= symbol_width;

                                if symbol_width > remaining_width {
                                    break;
                                }
                                remaining_width -= symbol_width;
                                first_whitespace = unfinished_whitespaces.pop_front();
                            }
                            // In case all whitespaces have been exhausted
                            if symbol_whitespace && first_whitespace.is_none() {
                                // Prevent first whitespace to count towards next word
                                continue;
                            }
                        }

                        // Append symbol to unfinished, partially processed word
                        if symbol_whitespace {
                            whitespace_width += symbol_width;
                            unfinished_whitespaces.push_back(StyledGrapheme { symbol, style });
                        } else {
                            word_width += symbol_width;
                            unfinished_word.push(StyledGrapheme { symbol, style });
                        }

                        has_seen_non_whitespace = !symbol_whitespace;
                    }

                    // Append remaining text parts
                    if !unfinished_word.is_empty() || !unfinished_whitespaces.is_empty() {
                        if current_line.is_empty() && unfinished_word.is_empty() {
                            wrapped_lines.push(vec![]);
                        } else if !self.trim || !current_line.is_empty() {
                            current_line.extend(unfinished_whitespaces.into_iter());
                        }
                        current_line.append(&mut unfinished_word);
                    }
                    if !current_line.is_empty() {
                        wrapped_lines.push(current_line);
                    }
                    if wrapped_lines.is_empty() {
                        // Append empty line if there was nothing to wrap in the first place
                        wrapped_lines.push(vec![]);
                    }

                    self.wrapped_lines = Some(wrapped_lines.into_iter());
                } else {
                    // No more whole lines available -> stop repeatedly retrieving next wrapped line
                    break;
                }
            }
        }

        if let Some(line) = current_line {
            self.current_line = line;
            Some((&self.current_line[..], line_width, self.current_alignment))
        } else {
            None
        }
    }
}
