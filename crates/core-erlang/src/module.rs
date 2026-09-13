use core::fmt::{self, Write as _};

use crate::stringify_all;

use super::{Expr, FunctionName, Literal};

#[derive(Debug, Clone, PartialEq)]
pub struct LocalFunctionDef {
    pub name: FunctionName,
    pub fun: Expr,
}

impl fmt::Display for LocalFunctionDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} = {}", self.name, self.fun)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Attribute {
    pub name: String,
    pub value: Literal,
}

impl fmt::Display for Attribute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "'{}' = {}", self.name, self.value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    pub name: String,
    pub exports: Vec<FunctionName>,
    pub attributes: Vec<Attribute>,
    pub functions: Vec<LocalFunctionDef>,
}

impl Module {
    pub fn builder(name: impl Into<String>) -> ModuleBuilder {
        ModuleBuilder {
            name: name.into(),
            exports: Vec::new(),
            attributes: Vec::new(),
            functions: Vec::new(),
        }
    }
}

impl fmt::Display for Module {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut out = format!(
            "module '{}' [{}]\n",
            self.name,
            stringify_all(&self.exports).join(", ")
        );

        if !self.attributes.is_empty() {
            let _ = write!(
                out,
                "  attributes [{}]\n\n",
                stringify_all(&self.attributes).join(", ")
            );
        }

        for func in &self.functions {
            let _ = write!(out, "{func}\n\n");
        }

        out.push_str("end");
        write!(f, "{out}")
    }
}

pub struct ModuleBuilder {
    name: String,
    exports: Vec<FunctionName>,
    attributes: Vec<Attribute>,
    functions: Vec<LocalFunctionDef>,
}

impl ModuleBuilder {
    #[must_use]
    pub fn export(mut self, name: impl Into<String>, arity: usize) -> Self {
        self.exports.push(FunctionName {
            name: name.into(),
            arity,
        });

        self
    }

    #[must_use]
    pub fn attribute(mut self, name: impl Into<String>, value: Literal) -> Self {
        self.attributes.push(Attribute {
            name: name.into(),
            value,
        });

        self
    }

    #[must_use]
    pub fn function(mut self, name: impl Into<String>, arity: usize, body: Expr) -> Self {
        self.functions.push(LocalFunctionDef {
            name: FunctionName {
                name: name.into(),
                arity,
            },
            fun: body,
        });

        self
    }

    #[must_use]
    pub fn function_and_export(
        self,
        name: impl Into<String> + Clone,
        arity: usize,
        body: Expr,
    ) -> Self {
        self.export(name.clone(), arity).function(name, arity, body)
    }

    pub fn build(self) -> Module {
        Module {
            name: self.name,
            exports: self.exports,
            attributes: self.attributes,
            functions: self.functions,
        }
    }
}
