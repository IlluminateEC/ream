#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(non_camel_case_types)]
#[repr(u16)]
pub enum SyntaxKind {
    /// Emitted when the lexer is unable to handle a grapheme
    BAD_TOKEN = 0,

    // Trivia
    WHITESPACE,
    /// `// a comment that is ignored`
    COMMENT,
    /// `/// documentation for an item`
    DOC_COMMENT,
    /// `//! documentation for the parent item`
    SUPER_DOC_COMMENT,

    IDENTIFIER,
    ATOM,
    INTEGER,
    FRACTIONAL,
    BOOLEAN,
    ERROR,

    // Punctuation
    COLON,
    DOT,
    COMMA,
    AT,
    ARROW,
    FAT_ARROW,
    // HASH,
    EQUAL,

    // Parenthesis
    LBRACE,
    RBRACE,
    LPAREN,
    RPAREN,
    LBRACKET,
    RBRACKET,
    MAP_BRACE,

    // Operators
    PLUS,
    HYPHEN,
    ASTERISK,
    SLASH,
    PERCENT,
    EXCLAMATION_POINT,
    PIPE,
    AMPERSAND,

    // Keywords
    AS,
    FN,
    FOR,
    LET,
    IMPL,
    TYPE,
    MATCH,
    TRAIT,
    IMPORT,

    // Derived Nodes
    OPERATION,
    MAP,
    MAP_PAIR,
    TUPLE,
    TUPLE_ITEM,
    GENERIC_INTRODUCTION,
    GENERIC_ARGS,
    GENERIC_ARG,
    FN_ARG,
    FN_ARGS,
    TYPE_UNION,
    TYPE_INTERSECTION,
    ROOT,
}

// TODO: <<, >>, <, <=, >, >=, ==, !=

impl SyntaxKind {
    pub const fn is_trivia(&self) -> bool {
        use SyntaxKind::*;

        matches!(self, WHITESPACE | COMMENT | DOC_COMMENT | SUPER_DOC_COMMENT)
    }

    pub const fn is_error(self) -> bool {
        use SyntaxKind::*;

        matches!(self, BAD_TOKEN | ERROR)
    }
}

impl From<SyntaxKind> for rowan::SyntaxKind {
    fn from(kind: SyntaxKind) -> Self {
        Self(kind as u16)
    }
}

impl From<rowan::SyntaxKind> for SyntaxKind {
    fn from(raw: rowan::SyntaxKind) -> Self {
        assert!(raw.0 <= Self::ROOT as u16);

        // SAFETY: we have checked that it is within the bounds of SyntaxKind.
        unsafe { std::mem::transmute::<u16, Self>(raw.0) }
    }
}
