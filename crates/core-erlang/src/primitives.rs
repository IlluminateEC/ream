use core::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VarName(pub String);

impl fmt::Display for VarName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionName {
    pub name: String,
    pub arity: usize,
}

impl fmt::Display for FunctionName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "'{}'/{}", self.name, self.arity)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Atom(String),
    Integer(i64),
    Float(f64),
    Char(char),
    Nil,
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Atom(name) => write!(f, "'{name}'"),
            Self::Integer(value) => write!(f, "{value}"),
            Self::Float(value) => write!(f, "{value}"),
            Self::Char(character) => write!(f, "${character}"),
            Self::Nil => write!(f, "[]"),
        }
    }
}
