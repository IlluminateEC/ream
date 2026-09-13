use core::fmt;
use std::string::ToString;

use crate::stringify_all;

use super::{BitSegment, CaseClause, FunctionName, Literal, LocalFunctionDef, VarName};

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Var(VarName),
    Literal(Literal),
    FunctionName(FunctionName),
    Tuple(Vec<Self>),
    List {
        head: Box<Self>,
        tail: Box<Self>,
    },
    Binary(Vec<BitSegment<Self>>),
    Values(Vec<Self>),

    // Function abstraction
    Fun {
        parameters: Vec<VarName>,
        body: Box<Self>,
    },

    // Bindings
    Let {
        variables: Vec<VarName>,
        value: Box<Self>,
        body: Box<Self>,
    },
    LetRec {
        definitions: Vec<LocalFunctionDef>,
        body: Box<Self>,
    },

    // Control Flow
    Case {
        expression: Box<Self>,
        clauses: Vec<CaseClause>,
    },
    Receive {
        clauses: Vec<CaseClause>,
        timeout_duration: Box<Self>,
        timeout_body: Box<Self>,
    },
    Try {
        expression: Box<Self>,
        success_variables: Vec<VarName>,
        success_body: Box<Self>,
        catch_variables: Vec<VarName>,
        catch_body: Box<Self>,
    },

    // Calls
    Apply {
        callee: Box<Self>,
        arguments: Vec<Self>,
    },
    Call {
        module: Box<Self>,
        function: Box<Self>,
        arguments: Vec<Self>,
    },
    PrimOp {
        name: String,
        arguments: Vec<Self>,
    },

    // Annotations
    Annotated {
        expr: Box<Self>,
        annotations: Vec<Literal>,
    },
}

impl Expr {
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

    pub fn fun(parameters: Vec<&str>, body: Self) -> Self {
        Self::Fun {
            parameters: parameters
                .into_iter()
                .map(|p| VarName(p.to_string()))
                .collect(),
            body: Box::new(body),
        }
    }

    pub fn let_bind(variables: Vec<&str>, value: Self, body: Self) -> Self {
        Self::Let {
            variables: variables
                .into_iter()
                .map(ToString::to_string)
                .map(VarName)
                .collect(),
            value: Box::new(value),
            body: Box::new(body),
        }
    }

    pub fn call(module: Self, function: Self, arguments: Vec<Self>) -> Self {
        Self::Call {
            module: Box::new(module),
            function: Box::new(function),
            arguments,
        }
    }

    pub fn apply(callee: Self, arguments: Vec<Self>) -> Self {
        Self::Apply {
            callee: Box::new(callee),
            arguments,
        }
    }

    pub fn case_expr(expr: Self, clauses: Vec<CaseClause>) -> Self {
        Self::Case {
            expression: Box::new(expr),
            clauses,
        }
    }
}

impl fmt::Display for Expr {
    #[allow(clippy::too_many_lines)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Var(v) => write!(f, "{v}"),

            Self::Literal(l) => write!(f, "{l}"),

            Self::FunctionName(fname) => write!(f, "{fname}"),

            Self::Tuple(elems) => {
                write!(f, "{{{}}}", stringify_all(elems).join(", "))
            }

            Self::List { head, tail } => write!(f, "[{head} | {tail}]"),

            Self::Binary(segments) => {
                write!(f, "#{{ {} }}#", stringify_all(segments).join(", "))
            }

            Self::Values(values) => {
                write!(f, "<{}>", stringify_all(values).join(", "))
            }

            Self::Fun { parameters, body } => {
                write!(
                    f,
                    "fun ({}) ->\n    {}",
                    stringify_all(parameters).join(", "),
                    body
                )
            }

            Self::Let {
                variables,
                value,
                body,
            } => {
                write!(
                    f,
                    "let <{}> = {} in\n  {}",
                    stringify_all(variables).join(", "),
                    value,
                    body
                )
            }

            Self::LetRec { definitions, body } => {
                write!(
                    f,
                    "letrec\n  {}\nin\n  {}",
                    stringify_all(definitions).join("\n  "),
                    body
                )
            }

            Self::Case {
                expression,
                clauses,
            } => {
                write!(
                    f,
                    "case {} of\n{}\nend",
                    expression,
                    stringify_all(clauses).join("\n")
                )
            }

            Self::Receive {
                clauses,
                timeout_duration,
                timeout_body,
            } => {
                write!(
                    f,
                    "receive\n{}\nafter {} ->\n  {}",
                    stringify_all(clauses).join("\n"),
                    timeout_duration,
                    timeout_body
                )
            }

            Self::Try {
                expression,
                success_variables,
                success_body,
                catch_variables,
                catch_body,
            } => {
                write!(
                    f,
                    "try {}\nof <{}> ->\n  {}\ncatch <{}> ->\n  {}",
                    expression,
                    stringify_all(success_variables).join(", "),
                    success_body,
                    stringify_all(catch_variables).join(", "),
                    catch_body
                )
            }

            Self::Apply { callee, arguments } => {
                write!(
                    f,
                    "apply {}({})",
                    callee,
                    stringify_all(arguments).join(", ")
                )
            }

            Self::Call {
                module,
                function,
                arguments,
            } => {
                write!(
                    f,
                    "call {}:{}({})",
                    module,
                    function,
                    stringify_all(arguments).join(", ")
                )
            }

            Self::PrimOp { name, arguments } => {
                write!(
                    f,
                    "primop '{}'({})",
                    name,
                    stringify_all(arguments).join(", ")
                )
            }

            Self::Annotated { expr, annotations } => {
                write!(
                    f,
                    "( {} -| [{}] )",
                    expr,
                    stringify_all(annotations).join(", ")
                )
            }
        }
    }
}
