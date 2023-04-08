use std::ops::Range;

use codespan_reporting::diagnostic::Label;

/// A span object that represents a slice of text in a file.
///
/// This object is designed to be used in conjunction with the `codespan` crate.
/// Through the [`Span::primary_label`] and [`Span::secondary_label`] methods,
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    start: usize, // TODO: benchmark the performance using u32 instead of usize
    end: usize,
    file_id: usize,
}

impl Span {
    pub fn new(start: usize, end: usize, file_id: usize) -> Self {
        debug_assert!(start <= end, "Span start must be before span end");
        Self {
            start,
            end,
            file_id,
        }
    }

    pub fn from_range(range: Range<usize>, file_id: usize) -> Self {
        Self {
            start: range.start,
            end: range.end,
            file_id,
        }
    }

    /// Creates a new Label from a span.
    pub fn primary_label(&self, message: impl Into<String>) -> Label<usize> {
        Label::primary(self.file_id, self.range()).with_message(message)
    }

    /// Creates a new Label from a span.
    pub fn secondary_label(&self, message: impl Into<String>) -> Label<usize> {
        Label::secondary(self.file_id, self.range()).with_message(message)
    }

    /// Creates a new Span that goes from start of s1 to the end of s2
    pub fn merge(s1: Self, s2: Self) -> Self {
        debug_assert_eq!(s1.file_id, s2.file_id);
        Self::new(s1.start, s2.end, s1.file_id)
    }

    /// Returns the range start..end
    pub fn range(&self) -> Range<usize> {
        self.start..self.end
    }

    /// Returns the start of the span
    pub fn start(&self) -> usize {
        self.start
    }

    /// Returns the end of the span
    pub fn end(&self) -> usize {
        self.end
    }

    /// Returns the file id of the span
    pub fn file_id(&self) -> usize {
        self.file_id
    }
}
