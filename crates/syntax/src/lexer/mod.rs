pub mod span;

use std::hint::unreachable_unchecked;

use crate::{
    lexer::span::{Span, Token},
    syntax_kind::SyntaxKind,
};

macro_rules! match_characters {
    ($self:ident ($current_character:expr) { $($character:literal => $variant:ident,)* }) => {
        match $current_character {
            $(Some($character) => {
                $self.cursor.bump_for_character($character);

                return Some($self.make_token($self.cursor, SyntaxKind::$variant));
            })*

            _ => (),
        }
    };
}

macro_rules! match_strings {
    ($self:ident { $($string:literal => $variant:ident,)* }) => {
        $(if $self.cursor.has_string($string) {
            $self.cursor.bump_for_string($string);

            return Some($self.make_token($self.cursor, SyntaxKind::$variant));
        })*
    };
}

pub struct Lexer<'source> {
    cursor: Span<'source>,
}

impl<'source> Lexer<'source> {
    pub const fn new(body: &'source str) -> Self {
        Self {
            cursor: Span::new(body),
        }
    }

    /// # Safety
    /// [`grapheme`] cannot be an empty string.
    fn starts_identifier(grapheme: &str) -> bool {
        // SAFETY: there is no way this can be None here
        let first_character = unsafe { grapheme.chars().next().unwrap_unchecked() };

        first_character == '_' || first_character.is_alphabetic() || emojis::get(grapheme).is_some()
    }

    /// # Safety
    /// [`grapheme`] cannot be an empty string.
    fn continues_identifier(grapheme: &str) -> bool {
        // SAFETY: there is no way this can be None here
        let first_character = unsafe { grapheme.chars().next().unwrap_unchecked() };

        first_character == '_'
            || first_character.is_alphabetic()
            || first_character.is_ascii_digit()
            || emojis::get(grapheme).is_some()
    }

    fn map_keyword(contents: &str) -> SyntaxKind {
        match &contents.to_lowercase() as &str {
            "as" => SyntaxKind::AS,
            "fn" => SyntaxKind::FN,
            "if" => SyntaxKind::IF,
            "for" => SyntaxKind::FOR,
            "let" => SyntaxKind::LET,
            "impl" => SyntaxKind::IMPL,
            "type" => SyntaxKind::TYPE,
            "match" => SyntaxKind::MATCH,
            "trait" => SyntaxKind::TRAIT,
            "import" => SyntaxKind::IMPORT,

            "true" | "false" => SyntaxKind::BOOLEAN,

            _ => SyntaxKind::IDENTIFIER,
        }
    }

    pub fn make_token(&mut self, span: Span<'source>, kind: SyntaxKind) -> Token<'source> {
        let token = Token {
            kind,
            contents: span.as_str(),
        };

        self.cursor.yank();

        token
    }
}

impl<'source> Iterator for Lexer<'source> {
    type Item = Token<'source>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(whitespace) = self
            .cursor
            .take_while_and_require_advancement(char::is_whitespace)
        {
            return Some(self.make_token(whitespace, SyntaxKind::WHITESPACE));
        }

        if self.cursor.current_character() == Some(':') {
            self.cursor.bump_for_character(':');

            return Some(
                if let Some(current_grapheme) = self.cursor.current_grapheme()
                    && Self::continues_identifier(current_grapheme)
                {
                    self.cursor.bump_for_string(current_grapheme);

                    self.cursor.take_while_grapheme(Self::continues_identifier);

                    self.make_token(self.cursor, SyntaxKind::ATOM)
                } else {
                    self.make_token(self.cursor, SyntaxKind::COLON)
                },
            );
        }

        if let Some(current_character) = self.cursor.current_character()
            && current_character.is_ascii_digit()
        {
            let Some(integer) = self
                .cursor
                .take_while_and_require_advancement(|character| character.is_ascii_digit())
            else {
                // SAFETY: we just checked if there was at least one ascii digit.
                unsafe { unreachable_unchecked() }
            };

            let fractional = if self.cursor.has_character('.') {
                self.cursor.bump_for_character('.');

                let fractional = self
                    .cursor
                    .take_while_and_require_advancement(|character| character.is_ascii_digit());

                if fractional.is_none() {
                    // Unconsume `.`
                    self.cursor.bump_by(-1);
                }

                fractional
            } else {
                None
            };

            return if let Some(fractional) = fractional {
                Some(self.make_token(integer.to(fractional), SyntaxKind::FRACTIONAL))
            } else {
                Some(self.make_token(integer, SyntaxKind::INTEGER))
            };
        }

        if let Some(current_grapheme) = self.cursor.current_grapheme()
            && Self::starts_identifier(current_grapheme)
        {
            self.cursor.bump_for_string(current_grapheme);

            self.cursor.take_while_grapheme(Self::continues_identifier);

            return Some(self.make_token(self.cursor, Self::map_keyword(self.cursor.as_str())));
        }

        if self.cursor.has_string("//") {
            self.cursor.bump_for_string("//");
            self.cursor.take_while(|character| character != '\n');

            if self.cursor.current_character() == Some('\n') {
                self.cursor.bump_for_character('\n');
            }

            return Some(self.make_token(
                self.cursor,
                match self.cursor.as_str().chars().nth(2) {
                    Some('/') => SyntaxKind::DOC_COMMENT,
                    Some('!') => SyntaxKind::SUPER_DOC_COMMENT,
                    _ => SyntaxKind::COMMENT,
                },
            ));
        }

        match_strings! {
            self {
                "->" => ARROW,
                "=>" => FAT_ARROW,

                "#{" => MAP_BRACE,
            }
        }

        match_characters! {
            self (self.cursor.current_character()) {
                '{' => LBRACE,
                '}' => RBRACE,

                '(' => LPAREN,
                ')' => RPAREN,

                '[' => LBRACKET,
                ']' => RBRACKET,

                '.' => DOT,
                ',' => COMMA,
                '@' => AT,
                // '#' => HASH,
                '=' => EQUAL,

                '+' => PLUS,
                '-' => HYPHEN,
                '*' => ASTERISK,
                '/' => SLASH,
                '%' => PERCENT,
                '!' => EXCLAMATION_POINT,
                '|' => PIPE,
                '&' => AMPERSAND,
            }
        }

        if let Some(current_grapheme) = self.cursor.current_grapheme() {
            self.cursor.bump_for_string(current_grapheme);

            Some(self.make_token(self.cursor, SyntaxKind::BAD_TOKEN))
        } else {
            None
        }
    }
}
