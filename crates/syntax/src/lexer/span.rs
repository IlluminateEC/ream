use unicode_segmentation::UnicodeSegmentation;

use crate::syntax_kind::SyntaxKind;

#[derive(Clone, Copy)]
pub struct Span<'source> {
    body: &'source str,

    start: usize,
    end: usize,
}

#[derive(Clone, Copy)]
pub struct Token<'source> {
    pub kind: SyntaxKind,
    pub contents: &'source str,
}

impl Token<'_> {
    pub const fn is_trivia(&self) -> bool {
        self.kind.is_trivia()
    }

    pub const fn is_error(&self) -> bool {
        self.kind.is_error()
    }
}

#[allow(clippy::arithmetic_side_effects)]
impl<'source> Span<'source> {
    pub const fn new(body: &'source str) -> Self {
        Self {
            body,

            start: 0,
            end: 0,
        }
    }

    pub fn current_character(&self) -> Option<char> {
        self.body[self.end..].chars().next()
    }

    pub fn has_character(&self, character: char) -> bool {
        self.current_character() == Some(character)
    }

    pub fn current_grapheme(&self) -> Option<&'source str> {
        self.body[self.end..].graphemes(true).next()
    }

    pub fn has_grapheme(&self, grapheme: &str) -> bool {
        self.current_grapheme() == Some(grapheme)
    }

    pub fn has_string(&self, sequence: &str) -> bool {
        self.body.get(self.end..self.end + sequence.len()) == Some(sequence)
    }

    pub const fn bump_by(&mut self, length: usize) {
        self.end += length;
    }

    pub const fn bump_for_string(&mut self, string: &str) {
        self.end += string.len();
    }

    pub const fn bump_for_character(&mut self, character: char) {
        self.end += character.len_utf8();
    }

    pub fn take_while(&mut self, predicate: impl Fn(char) -> bool) {
        while let Some(character) = self.current_character()
            && predicate(character)
        {
            self.bump_for_character(character);
        }
    }

    pub fn take_while_grapheme(&mut self, predicate: impl Fn(&str) -> bool) {
        while let Some(grapheme) = self.current_grapheme()
            && predicate(grapheme)
        {
            self.bump_for_string(grapheme);
        }
    }

    pub fn take_while_and_require_advancement(
        &mut self,
        predicate: impl Fn(char) -> bool,
    ) -> Option<&mut Self> {
        let original_end = self.end;

        self.take_while(predicate);

        if original_end == self.end {
            None
        } else {
            Some(self)
        }
    }

    pub const fn yank(&mut self) {
        self.start = self.end;
    }

    pub fn as_str(&self) -> &'source str {
        &self.body[self.start..self.end]
    }

    pub fn make_token(&mut self, kind: SyntaxKind) -> (SyntaxKind, &'source str) {
        let result = (kind, self.as_str());

        self.yank();

        result
    }

    pub const fn is_at_end(&self) -> bool {
        self.end >= self.body.len()
    }
}
