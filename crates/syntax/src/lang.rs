use crate::syntax_kind::SyntaxKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReamSyntax;

impl rowan::Language for ReamSyntax {
    type Kind = SyntaxKind;

    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        raw.into()
    }

    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        kind.into()
    }
}

pub type SyntaxNode = rowan::SyntaxNode<ReamSyntax>;

pub type SyntaxToken = rowan::SyntaxToken<ReamSyntax>;

pub type SyntaxElement = rowan::NodeOrToken<SyntaxNode, SyntaxToken>;
