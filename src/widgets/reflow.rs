use std::{collections::VecDeque, vec::IntoIter};

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::{layout::Alignment, text::StyledGrapheme};

/// A state machine to pack styled symbols into lines.
/// Cannot implement it as Iterator since it yields slices of the internal buffer (need streaming
/// iterators for that).
pub trait LineComposer<'a> {
    fn next_line<'lend>(&'lend mut self) -> Option<WrappedLine<'lend, 'a>>;
}

pub struct WrappedLine<'lend, 'text> {
    /// One line reflowed to the correct width
    pub line: &'lend [StyledGrapheme<'text>],
    /// The width of the line
    pub width: u16,
    /// Whether the line was aligned left or right
    pub alignment: Alignment,
}

/// A state machine that wraps lines on word boundaries.
#[derive(Debug, Default, Clone)]
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
    pub fn new(lines: O, max_line_width: u16, trim: bool) -> Self {
        Self {
            input_lines: lines,
            max_line_width,
            wrapped_lines: None,
            current_alignment: Alignment::Left,
            current_line: vec![],
            trim,
        }
    }

    fn next_cached_line(&mut self) -> Option<Vec<StyledGrapheme<'a>>> {
        self.wrapped_lines.as_mut()?.next()
    }

    fn process_input(&mut self, line_symbols: impl IntoIterator<Item = StyledGrapheme<'a>>) {
        let mut result = vec![];
        let mut current_line = vec![];
        let mut current_line_width = 0;
        let mut pending_word = vec![];
        let mut word_width = 0;
        let mut pending_whitespace: VecDeque<StyledGrapheme> = VecDeque::new();
        let mut whitespace_width = 0;
        let mut non_whitespace_previous = false;

        for grapheme in line_symbols {
            let is_whitespace = grapheme.is_whitespace();
            let symbol_width = grapheme.symbol.width() as u16;

            // ignore symbols wider than max width
            if symbol_width > self.max_line_width {
                continue;
            }

            let word_found = non_whitespace_previous && is_whitespace;
            // current word would overflow after removing whitespace
            let trimmed_overflow = word_width + symbol_width > self.max_line_width
                && current_line.is_empty()
                && self.trim;
            // separated whitespace would overflow on its own
            let whitespace_overflow = whitespace_width + symbol_width > self.max_line_width
                && current_line.is_empty()
                && self.trim;
            // current full word (including whitespace) would overflow
            let untrimmed_overflow = word_width + whitespace_width + symbol_width
                > self.max_line_width
                && current_line.is_empty()
                && !self.trim;

            // append finished segment to current line
            if word_found || trimmed_overflow || whitespace_overflow || untrimmed_overflow {
                if !current_line.is_empty() || !self.trim {
                    current_line.extend(pending_whitespace.drain(..));
                    current_line_width += whitespace_width;
                }

                current_line.append(&mut pending_word);
                current_line_width += word_width;

                pending_whitespace.clear();
                whitespace_width = 0;
                word_width = 0;
            }

            // add finished wrapped line to remaining lines
            if current_line_width >= self.max_line_width
                || current_line_width + whitespace_width + word_width >= self.max_line_width
                    && symbol_width > 0
            {
                let mut remaining_width =
                    u16::saturating_sub(self.max_line_width, current_line_width);

                result.push(std::mem::take(&mut current_line));
                current_line_width = 0;

                // remove whitespace up to the end of line
                while let Some(grapheme) = pending_whitespace.front() {
                    let width = grapheme.symbol.width() as u16;

                    if width > remaining_width {
                        break;
                    }

                    whitespace_width -= width;
                    remaining_width -= width;
                    pending_whitespace.pop_front();
                }

                // don't count first whitespace toward next word
                if is_whitespace && pending_whitespace.is_empty() {
                    continue;
                }
            }

            // append symbol to a pending buffer
            if is_whitespace {
                whitespace_width += symbol_width;
                pending_whitespace.push_back(grapheme);
            } else {
                word_width += symbol_width;
                pending_word.push(grapheme);
            }

            non_whitespace_previous = !is_whitespace;
        }

        // append remaining text parts
        if !pending_word.is_empty() || !pending_whitespace.is_empty() {
            if current_line.is_empty() && pending_word.is_empty() {
                result.push(vec![]);
            } else if !self.trim || !current_line.is_empty() {
                current_line.extend(pending_whitespace);
            } else {
                // TODO: explain why this else branch is ok
                // See clippy::else_if_without_else
            }

            current_line.append(&mut pending_word);
        }
        if !current_line.is_empty() {
            result.push(current_line);
        }
        if result.is_empty() {
            result.push(vec![]);
        }

        // save cached lines for emitting later
        self.wrapped_lines = Some(result.into_iter());
    }
}

impl<'a, O, I> LineComposer<'a> for WordWrapper<'a, O, I>
where
    O: Iterator<Item = (I, Alignment)>,
    I: Iterator<Item = StyledGrapheme<'a>>,
{
    #[allow(clippy::too_many_lines)]
    fn next_line<'lend>(&'lend mut self) -> Option<WrappedLine<'lend, 'a>> {
        if self.max_line_width == 0 {
            return None;
        }

        loop {
            // emit next cached line if present
            if let Some(line) = self.next_cached_line() {
                let line_width = line
                    .iter()
                    .map(|grapheme| grapheme.symbol.width() as u16)
                    .sum();

                self.current_line = line;
                return Some(WrappedLine {
                    line: &self.current_line,
                    width: line_width,
                    alignment: self.current_alignment,
                });
            }

            // otherwise, process pending wrapped lines from input
            let (line_symbols, line_alignment) = self.input_lines.next()?;
            self.current_alignment = line_alignment;
            self.process_input(line_symbols);
        }
    }
}

/// A state machine that truncates overhanging lines.
#[derive(Debug, Default, Clone)]
pub struct LineTruncator<'a, O, I>
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
    current_line: Vec<StyledGrapheme<'a>>,
    /// Record the offset to skip render
    horizontal_offset: u16,
}

impl<'a, O, I> LineTruncator<'a, O, I>
where
    O: Iterator<Item = (I, Alignment)>,
    I: Iterator<Item = StyledGrapheme<'a>>,
{
    pub fn new(lines: O, max_line_width: u16) -> Self {
        Self {
            input_lines: lines,
            max_line_width,
            horizontal_offset: 0,
            current_line: vec![],
        }
    }

    pub fn set_horizontal_offset(&mut self, horizontal_offset: u16) {
        self.horizontal_offset = horizontal_offset;
    }
}

impl<'a, O, I> LineComposer<'a> for LineTruncator<'a, O, I>
where
    O: Iterator<Item = (I, Alignment)>,
    I: Iterator<Item = StyledGrapheme<'a>>,
{
    fn next_line<'lend>(&'lend mut self) -> Option<WrappedLine<'lend, 'a>> {
        if self.max_line_width == 0 {
            return None;
        }

        self.current_line.truncate(0);
        let mut current_line_width = 0;

        let mut lines_exhausted = true;
        let mut horizontal_offset = self.horizontal_offset as usize;
        let mut current_alignment = Alignment::Left;
        if let Some((current_line, alignment)) = &mut self.input_lines.next() {
            lines_exhausted = false;
            current_alignment = *alignment;

            for StyledGrapheme { symbol, style } in current_line {
                // Ignore characters wider that the total max width.
                if symbol.width() as u16 > self.max_line_width {
                    continue;
                }

                if current_line_width + symbol.width() as u16 > self.max_line_width {
                    // Truncate line
                    break;
                }

                let symbol = if horizontal_offset == 0 || Alignment::Left != *alignment {
                    symbol
                } else {
                    let w = symbol.width();
                    if w > horizontal_offset {
                        let t = trim_offset(symbol, horizontal_offset);
                        horizontal_offset = 0;
                        t
                    } else {
                        horizontal_offset -= w;
                        ""
                    }
                };
                current_line_width += symbol.width() as u16;
                self.current_line.push(StyledGrapheme { symbol, style });
            }
        }

        if lines_exhausted {
            None
        } else {
            Some(WrappedLine {
                line: &self.current_line,
                width: current_line_width,
                alignment: current_alignment,
            })
        }
    }
}

/// This function will return a str slice which start at specified offset.
/// As src is a unicode str, start offset has to be calculated with each character.
fn trim_offset(src: &str, mut offset: usize) -> &str {
    let mut start = 0;
    for c in UnicodeSegmentation::graphemes(src, true) {
        let w = c.width();
        if w <= offset {
            offset -= w;
            start += c.len();
        } else {
            break;
        }
    }
    #[allow(clippy::string_slice)] // Is safe as it comes from UnicodeSegmentation
    &src[start..]
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        style::Style,
        text::{Line, Text},
    };

    #[derive(Clone, Copy)]
    enum Composer {
        WordWrapper { trim: bool },
        LineTruncator,
    }

    fn run_composer<'a>(
        which: Composer,
        text: impl Into<Text<'a>>,
        text_area_width: u16,
    ) -> (Vec<String>, Vec<u16>, Vec<Alignment>) {
        let text = text.into();
        let styled_lines = text.iter().map(|line| {
            (
                line.iter()
                    .flat_map(|span| span.styled_graphemes(Style::default())),
                line.alignment.unwrap_or(Alignment::Left),
            )
        });

        let mut composer: Box<dyn LineComposer> = match which {
            Composer::WordWrapper { trim } => {
                Box::new(WordWrapper::new(styled_lines, text_area_width, trim))
            }
            Composer::LineTruncator => Box::new(LineTruncator::new(styled_lines, text_area_width)),
        };
        let mut lines = vec![];
        let mut widths = vec![];
        let mut alignments = vec![];
        while let Some(WrappedLine {
            line: styled,
            width,
            alignment,
        }) = composer.next_line()
        {
            let line = styled
                .iter()
                .map(|StyledGrapheme { symbol, .. }| *symbol)
                .collect::<String>();
            assert!(width <= text_area_width);
            lines.push(line);
            widths.push(width);
            alignments.push(alignment);
        }
        (lines, widths, alignments)
    }

    #[test]
    fn line_composer_one_line() {
        let width = 40;
        for i in 1..width {
            let text = "a".repeat(i);
            let (word_wrapper, _, _) =
                run_composer(Composer::WordWrapper { trim: true }, &*text, width as u16);
            let (line_truncator, _, _) =
                run_composer(Composer::LineTruncator, &*text, width as u16);
            let expected = vec![text];
            assert_eq!(word_wrapper, expected);
            assert_eq!(line_truncator, expected);
        }
    }

    #[test]
    fn line_composer_short_lines() {
        let width = 20;
        let text =
            "abcdefg\nhijklmno\npabcdefg\nhijklmn\nopabcdefghijk\nlmnopabcd\n\n\nefghijklmno";
        let (word_wrapper, _, _) = run_composer(Composer::WordWrapper { trim: true }, text, width);
        let (line_truncator, _, _) = run_composer(Composer::LineTruncator, text, width);

        let wrapped: Vec<&str> = text.split('\n').collect();
        assert_eq!(word_wrapper, wrapped);
        assert_eq!(line_truncator, wrapped);
    }

    #[test]
    fn line_composer_long_word() {
        let width = 20;
        let text = "abcdefghijklmnopabcdefghijklmnopabcdefghijklmnopabcdefghijklmno";
        let (word_wrapper, _, _) =
            run_composer(Composer::WordWrapper { trim: true }, text, width as u16);
        let (line_truncator, _, _) = run_composer(Composer::LineTruncator, text, width as u16);

        let wrapped = vec![
            text.get(..width).unwrap(),
            text.get(width..width * 2).unwrap(),
            text.get(width * 2..width * 3).unwrap(),
            text.get(width * 3..).unwrap(),
        ];
        assert_eq!(
            word_wrapper, wrapped,
            "WordWrapper should detect the line cannot be broken on word boundary and \
             break it at line width limit."
        );
        assert_eq!(line_truncator, [text.get(..width).unwrap()]);
    }

    #[test]
    fn line_composer_long_sentence() {
        let width = 20;
        let text =
            "abcd efghij klmnopabcd efgh ijklmnopabcdefg hijkl mnopab c d e f g h i j k l m n o";
        let text_multi_space =
            "abcd efghij    klmnopabcd efgh     ijklmnopabcdefg hijkl mnopab c d e f g h i j k l \
             m n o";
        let (word_wrapper_single_space, _, _) =
            run_composer(Composer::WordWrapper { trim: true }, text, width as u16);
        let (word_wrapper_multi_space, _, _) = run_composer(
            Composer::WordWrapper { trim: true },
            text_multi_space,
            width as u16,
        );
        let (line_truncator, _, _) = run_composer(Composer::LineTruncator, text, width as u16);

        let word_wrapped = vec![
            "abcd efghij",
            "klmnopabcd efgh",
            "ijklmnopabcdefg",
            "hijkl mnopab c d e f",
            "g h i j k l m n o",
        ];
        assert_eq!(word_wrapper_single_space, word_wrapped);
        assert_eq!(word_wrapper_multi_space, word_wrapped);

        assert_eq!(line_truncator, [text.get(..width).unwrap()]);
    }

    #[test]
    fn line_composer_zero_width() {
        let width = 0;
        let text = "abcd efghij klmnopabcd efgh ijklmnopabcdefg hijkl mnopab ";
        let (word_wrapper, _, _) = run_composer(Composer::WordWrapper { trim: true }, text, width);
        let (line_truncator, _, _) = run_composer(Composer::LineTruncator, text, width);

        let expected: Vec<&str> = Vec::new();
        assert_eq!(word_wrapper, expected);
        assert_eq!(line_truncator, expected);
    }

    #[test]
    fn line_composer_max_line_width_of_1() {
        let width = 1;
        let text = "abcd efghij klmnopabcd efgh ijklmnopabcdefg hijkl mnopab ";
        let (word_wrapper, _, _) = run_composer(Composer::WordWrapper { trim: true }, text, width);
        let (line_truncator, _, _) = run_composer(Composer::LineTruncator, text, width);

        let expected: Vec<&str> = UnicodeSegmentation::graphemes(text, true)
            .filter(|g| g.chars().any(|c| !c.is_whitespace()))
            .collect();
        assert_eq!(word_wrapper, expected);
        assert_eq!(line_truncator, vec!["a"]);
    }

    #[test]
    fn line_composer_max_line_width_of_1_double_width_characters() {
        let width = 1;
        let text =
            "コンピュータ上で文字を扱う場合、典型的には文字\naaa\naによる通信を行う場合にその\
                    両端点では、";
        let (word_wrapper, _, _) = run_composer(Composer::WordWrapper { trim: true }, text, width);
        let (line_truncator, _, _) = run_composer(Composer::LineTruncator, text, width);
        assert_eq!(word_wrapper, vec!["", "a", "a", "a", "a"]);
        assert_eq!(line_truncator, vec!["", "a", "a"]);
    }

    /// Tests `WordWrapper` with words some of which exceed line length and some not.
    #[test]
    fn line_composer_word_wrapper_mixed_length() {
        let width = 20;
        let text = "abcd efghij klmnopabcdefghijklmnopabcdefghijkl mnopab cdefghi j klmno";
        let (word_wrapper, _, _) = run_composer(Composer::WordWrapper { trim: true }, text, width);
        assert_eq!(
            word_wrapper,
            vec![
                "abcd efghij",
                "klmnopabcdefghijklmn",
                "opabcdefghijkl",
                "mnopab cdefghi j",
                "klmno",
            ]
        );
    }

    #[test]
    fn line_composer_double_width_chars() {
        let width = 20;
        let text = "コンピュータ上で文字を扱う場合、典型的には文字による通信を行う場合にその両端点\
                    では、";
        let (word_wrapper, word_wrapper_width, _) =
            run_composer(Composer::WordWrapper { trim: true }, text, width);
        let (line_truncator, _, _) = run_composer(Composer::LineTruncator, text, width);
        assert_eq!(line_truncator, vec!["コンピュータ上で文字"]);
        let wrapped = vec![
            "コンピュータ上で文字",
            "を扱う場合、典型的に",
            "は文字による通信を行",
            "う場合にその両端点で",
            "は、",
        ];
        assert_eq!(word_wrapper, wrapped);
        assert_eq!(word_wrapper_width, vec![width, width, width, width, 4]);
    }

    #[test]
    fn line_composer_leading_whitespace_removal() {
        let width = 20;
        let text = "AAAAAAAAAAAAAAAAAAAA    AAA";
        let (word_wrapper, _, _) = run_composer(Composer::WordWrapper { trim: true }, text, width);
        let (line_truncator, _, _) = run_composer(Composer::LineTruncator, text, width);
        assert_eq!(word_wrapper, vec!["AAAAAAAAAAAAAAAAAAAA", "AAA",]);
        assert_eq!(line_truncator, vec!["AAAAAAAAAAAAAAAAAAAA"]);
    }

    /// Tests truncation of leading whitespace.
    #[test]
    fn line_composer_lots_of_spaces() {
        let width = 20;
        let text = "                                                                     ";
        let (word_wrapper, _, _) = run_composer(Composer::WordWrapper { trim: true }, text, width);
        let (line_truncator, _, _) = run_composer(Composer::LineTruncator, text, width);
        assert_eq!(word_wrapper, vec![""]);
        assert_eq!(line_truncator, vec!["                    "]);
    }

    /// Tests an input starting with a letter, followed by spaces - some of the behaviour is
    /// incidental.
    #[test]
    fn line_composer_char_plus_lots_of_spaces() {
        let width = 20;
        let text = "a                                                                     ";
        let (word_wrapper, _, _) = run_composer(Composer::WordWrapper { trim: true }, text, width);
        let (line_truncator, _, _) = run_composer(Composer::LineTruncator, text, width);
        // What's happening below is: the first line gets consumed, trailing spaces discarded,
        // after 20 of which a word break occurs (probably shouldn't). The second line break
        // discards all whitespace. The result should probably be vec!["a"] but it doesn't matter
        // that much.
        assert_eq!(word_wrapper, vec!["a", ""]);
        assert_eq!(line_truncator, vec!["a                   "]);
    }

    #[test]
    fn line_composer_word_wrapper_double_width_chars_mixed_with_spaces() {
        let width = 20;
        // Japanese seems not to use spaces but we should break on spaces anyway... We're using it
        // to test double-width chars.
        // You are more than welcome to add word boundary detection based of alterations of
        // hiragana and katakana...
        // This happens to also be a test case for mixed width because regular spaces are single
        // width.
        let text = "コンピュ ータ上で文字を扱う場合、 典型的には文 字による 通信を行 う場合にその両端点では、";
        let (word_wrapper, word_wrapper_width, _) =
            run_composer(Composer::WordWrapper { trim: true }, text, width);
        assert_eq!(
            word_wrapper,
            vec![
                "コンピュ",
                "ータ上で文字を扱う場",
                "合、 典型的には文",
                "字による 通信を行",
                "う場合にその両端点で",
                "は、",
            ]
        );
        // Odd-sized lines have a space in them.
        assert_eq!(word_wrapper_width, vec![8, 20, 17, 17, 20, 4]);
    }

    /// Ensure words separated by nbsp are wrapped as if they were a single one.
    #[test]
    fn line_composer_word_wrapper_nbsp() {
        let width = 20;
        let text = "AAAAAAAAAAAAAAA AAAA\u{00a0}AAA";
        let (word_wrapper, word_wrapper_widths, _) =
            run_composer(Composer::WordWrapper { trim: true }, text, width);
        assert_eq!(word_wrapper, vec!["AAAAAAAAAAAAAAA", "AAAA\u{00a0}AAA",]);
        assert_eq!(word_wrapper_widths, vec![15, 8]);

        // Ensure that if the character was a regular space, it would be wrapped differently.
        let text_space = text.replace('\u{00a0}', " ");
        let (word_wrapper_space, word_wrapper_widths, _) =
            run_composer(Composer::WordWrapper { trim: true }, text_space, width);
        assert_eq!(word_wrapper_space, vec!["AAAAAAAAAAAAAAA AAAA", "AAA",]);
        assert_eq!(word_wrapper_widths, vec![20, 3]);
    }

    #[test]
    fn line_composer_word_wrapper_preserve_indentation() {
        let width = 20;
        let text = "AAAAAAAAAAAAAAAAAAAA    AAA";
        let (word_wrapper, _, _) = run_composer(Composer::WordWrapper { trim: false }, text, width);
        assert_eq!(word_wrapper, vec!["AAAAAAAAAAAAAAAAAAAA", "   AAA",]);
    }

    #[test]
    fn line_composer_word_wrapper_preserve_indentation_with_wrap() {
        let width = 10;
        let text = "AAA AAA AAAAA AA AAAAAA\n B\n  C\n   D";
        let (word_wrapper, _, _) = run_composer(Composer::WordWrapper { trim: false }, text, width);
        assert_eq!(
            word_wrapper,
            vec!["AAA AAA", "AAAAA AA", "AAAAAA", " B", "  C", "   D"]
        );
    }

    #[test]
    fn line_composer_word_wrapper_preserve_indentation_lots_of_whitespace() {
        let width = 10;
        let text = "               4 Indent\n                 must wrap!";
        let (word_wrapper, _, _) = run_composer(Composer::WordWrapper { trim: false }, text, width);
        assert_eq!(
            word_wrapper,
            vec![
                "          ",
                "    4",
                "Indent",
                "          ",
                "      must",
                "wrap!"
            ]
        );
    }

    #[test]
    fn line_composer_zero_width_at_end() {
        let width = 3;
        let line = "foo\u{200B}";
        let (word_wrapper, _, _) = run_composer(Composer::WordWrapper { trim: true }, line, width);
        let (line_truncator, _, _) = run_composer(Composer::LineTruncator, line, width);
        assert_eq!(word_wrapper, vec!["foo"]);
        assert_eq!(line_truncator, vec!["foo\u{200B}"]);
    }

    #[test]
    fn line_composer_preserves_line_alignment() {
        let width = 20;
        let lines = vec![
            Line::from("Something that is left aligned.").alignment(Alignment::Left),
            Line::from("This is right aligned and half short.").alignment(Alignment::Right),
            Line::from("This should sit in the center.").alignment(Alignment::Center),
        ];
        let (_, _, wrapped_alignments) =
            run_composer(Composer::WordWrapper { trim: true }, lines.clone(), width);
        let (_, _, truncated_alignments) = run_composer(Composer::LineTruncator, lines, width);
        assert_eq!(
            wrapped_alignments,
            vec![
                Alignment::Left,
                Alignment::Left,
                Alignment::Right,
                Alignment::Right,
                Alignment::Right,
                Alignment::Center,
                Alignment::Center
            ]
        );
        assert_eq!(
            truncated_alignments,
            vec![Alignment::Left, Alignment::Right, Alignment::Center]
        );
    }

    #[test]
    fn line_composer_zero_width_white_space() {
        let width = 3;
        let line = "foo\u{200b}bar";
        let (word_wrapper, _, _) = run_composer(Composer::WordWrapper { trim: true }, line, width);
        assert_eq!(word_wrapper, vec!["foo", "bar"]);
    }
}
