use core::fmt;

use crate::stringify_all;

use super::{Expr, Pattern};

#[derive(Debug, Clone, PartialEq)]
pub struct CaseClause {
    pub patterns: Vec<Pattern>,
    pub guard: Expr,
    pub body: Expr,
}

impl fmt::Display for CaseClause {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "  <{}> when {} ->\n    {}",
            stringify_all(&self.patterns).join(", "),
            self.guard,
            self.body
        )
    }
}
