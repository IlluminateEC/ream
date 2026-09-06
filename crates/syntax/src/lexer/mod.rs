pub mod span;

use crate::{lexer::span::Span, syntax_kind::SyntaxKind};

pub struct Lexer<'source> {
    span: Span<'source>,
}

impl<'source> Lexer<'source> {
    pub const fn new(body: &'source str) -> Self {
        Self {
            span: Span::new(body),
        }
    }

    fn starts_identifier(grapheme: &str) -> bool {
        // SAFETY: there is no way this can be None here
        let first_character = unsafe { grapheme.chars().next().unwrap_unchecked() };

        first_character == '_' || first_character.is_alphabetic() || emojis::get(grapheme).is_some()
    }

    fn continues_identifier(grapheme: &str) -> bool {
        // SAFETY: there is no way this can be None here
        let first_character = unsafe { grapheme.chars().next().unwrap_unchecked() };

        first_character == '_'
            || first_character.is_alphanumeric()
            || emojis::get(grapheme).is_some()
    }
}

impl<'source> Iterator for Lexer<'source> {
    type Item = (SyntaxKind, &'source str);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(whitespace) = self
            .span
            .take_while_and_require_advancement(char::is_whitespace)
        {
            return Some(whitespace.make_token(SyntaxKind::WHITESPACE));
        }

        if self.span.current_character() == Some(':') {
            self.span.bump_for_character(':');

            return Some(
                if let Some(current_grapheme) = self.span.current_grapheme()
                    && Self::continues_identifier(current_grapheme)
                {
                    self.span.bump_for_string(current_grapheme);

                    self.span.take_while_grapheme(Self::continues_identifier);

                    self.span.make_token(SyntaxKind::ATOM)
                } else {
                    self.span.make_token(SyntaxKind::COLON)
                },
            );
        }

        if let Some(current_grapheme) = self.span.current_grapheme()
            && Self::starts_identifier(current_grapheme)
        {
            self.span.bump_for_string(current_grapheme);

            self.span.take_while_grapheme(Self::continues_identifier);

            return Some(self.span.make_token(SyntaxKind::IDENTIFIER));
        }

        if let Some(current_grapheme) = self.span.current_grapheme() {
            self.span.bump_for_string(current_grapheme);

            Some(self.span.make_token(SyntaxKind::BAD_TOKEN))
        } else {
            None
        }
    }
}
