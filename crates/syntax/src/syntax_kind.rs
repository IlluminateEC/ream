#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(non_camel_case_types)]
#[repr(u16)]
pub enum SyntaxKind {
    BAD_TOKEN = 0,

    // Trivia
    WHITESPACE,
    COMMENT,
    DOC_COMMENT,

    IDENTIFIER,
    ATOM,
    ERROR,

    COLON,

    ADD,
    SUB,
    MUL,
    DIV,

    // Derived Nodes
    NUMBER,
    OPERATION,
    ROOT,
}

impl SyntaxKind {
    pub const fn is_trivia(&self) -> bool {
        match self {
            Self::WHITESPACE | Self::COMMENT | Self::DOC_COMMENT => true,
            Self::IDENTIFIER
            | Self::ATOM
            | Self::COLON
            | Self::ADD
            | Self::SUB
            | Self::MUL
            | Self::DIV
            | Self::NUMBER
            | Self::BAD_TOKEN
            | Self::ERROR
            | Self::OPERATION
            | Self::ROOT => false,
        }
    }

    pub const fn is_error(self) -> bool {
        matches!(self, Self::BAD_TOKEN | Self::ERROR)
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
