use core::fmt;

use super::{BitSegment, Literal, VarName, stringify_all};

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Var(VarName),
    Literal(Literal),
    Tuple(Vec<Self>),
    List {
        head: Box<Self>,
        tail: Box<Self>,
    },
    Binary(Vec<BitSegment<Self>>),
    Alias {
        binding: VarName,
        pattern: Box<Self>,
    },
}

impl Pattern {
    pub fn var(name: impl Into<String>) -> Self {
        Self::Var(VarName(name.into()))
    }

    pub fn atom(name: impl Into<String>) -> Self {
        Self::Literal(Literal::Atom(name.into()))
    }

    pub const fn int(value: i64) -> Self {
        Self::Literal(Literal::Integer(value))
    }

    pub const fn tuple(elements: Vec<Self>) -> Self {
        Self::Tuple(elements)
    }
}

impl fmt::Display for Pattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Var(name) => write!(f, "{name}"),
            Self::Literal(literal) => write!(f, "{literal}"),
            Self::Tuple(elements) => {
                write!(f, "{{{}}}", stringify_all(elements).join(", "))
            }
            Self::List { head, tail } => write!(f, "[{head} | {tail}]"),
            Self::Binary(segments) => {
                write!(f, "#{{ {} }}#", stringify_all(segments).join(", "))
            }
            Self::Alias { binding, pattern } => write!(f, "{binding} = {pattern}"),
        }
    }
}
