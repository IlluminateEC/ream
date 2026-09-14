use rowan::GreenNode;
use rowan::GreenNodeBuilder;

use crate::lexer::Lexer;
use crate::lexer::span::Token;
use crate::parser::combinator::ParseResult;
use crate::parser::combinator::ParserCombinator;
use crate::parser::combinator::ParserState;
use crate::syntax_kind::SyntaxKind;

#[derive(Debug, Clone)]
pub struct Parse {
    green_node: GreenNode,
    #[allow(unused)]
    errors: Vec<String>,
}

struct Parser<'source> {
    tokens: Vec<Token<'source>>,
    builder: GreenNodeBuilder<'static>,
    errors: Vec<String>,
}

impl ParserState for Parser<'_> {
    type TokenKind = SyntaxKind;

    fn peek_kind(&self) -> Option<Self::TokenKind> {
        self.peek().map(|token| token.kind)
    }

    fn is_at_eof(&self) -> bool {
        self.peek().is_none()
    }

    fn start_group(&mut self, kind: Self::TokenKind) {
        self.builder.start_node(kind.into());
    }

    fn end_group(&mut self) {
        self.builder.finish_node();
    }

    fn consume_trivia(&mut self) {
        while let Some(token) = self.peek()
            && token.is_trivia()
        {
            self.consume();
        }
    }

    /// Advance one token, adding it to the current branch of the tree builder.
    #[allow(clippy::arithmetic_side_effects)]
    fn consume(&mut self) {
        let Some(token) = self.tokens.pop() else {
            return;
        };

        self.builder.token(token.kind.into(), token.contents);
    }

    fn expect(&mut self, kind: Self::TokenKind) -> ParseResult {
        self.consume_trivia();

        self.expect_immediate(kind)
    }

    fn expect_immediate(&mut self, kind: Self::TokenKind) -> ParseResult {
        if let Some(token) = self.peek()
            && token.kind == kind
        {
            self.consume();

            ParseResult::Ok
        } else if self.peek().is_none() {
            ParseResult::Eof
        } else {
            ParseResult::UnexpectedToken
        }
    }
}

fn just<'source>(kind: SyntaxKind) -> ParserCombinator<'source, Parser<'source>> {
    ParserCombinator::just(kind)
}

impl<'source> Parser<'source> {
    fn parse(mut self) -> Parse {
        self.builder.start_node(SyntaxKind::ROOT.into());

        // parse top-level statments
        loop {
            match self.definition()(&mut self) {
                ParseResult::Eof => break,
                ParseResult::UnexpectedToken => {
                    self.builder.start_node(SyntaxKind::ERROR.into());
                    self.errors.push(format!(
                        "unexpected token: {}",
                        self.peek().map_or_else(
                            || "end of file".to_string(),
                            |token| format!("{token:?}")
                        )
                    ));
                    self.consume();
                    self.builder.finish_node();
                }
                ParseResult::Ok => (),
            }
        }

        self.consume_trivia();
        self.builder.finish_node();

        Parse {
            green_node: self.builder.finish(),
            errors: self.errors,
        }
    }

    /// Peek at the first unprocessed token
    fn peek(&self) -> Option<Token<'source>> {
        self.tokens.last().copied()
    }

    fn pattern(&self) -> ParserCombinator<'source, Self> {
        just(SyntaxKind::IDENTIFIER)
            .or(just(SyntaxKind::ATOM))
            .or(just(SyntaxKind::INTEGER))
            .group_as(SyntaxKind::PATTERN)
    }

    fn expression(&self) -> ParserCombinator<'source, Self> {
        ParserCombinator::recursive(|expression| {
            let let_expression = just(SyntaxKind::LET)
                .then(self.pattern())
                .then(just(SyntaxKind::COLON).then(self.r#type()).optional())
                .then(just(SyntaxKind::EQUAL))
                .then(expression.clone())
                .group_as(SyntaxKind::LET);

            let tuple = expression
                .clone()
                .group_as(SyntaxKind::TUPLE_ITEM)
                .repeated_with_mandatory_trailing_separator_for_one_item(
                    SyntaxKind::COMMA,
                    SyntaxKind::RBRACE,
                )
                .delimited(SyntaxKind::LBRACE, SyntaxKind::RBRACE)
                .group_as(SyntaxKind::TUPLE);

            let map_pair = just(SyntaxKind::IDENTIFIER)
                .or(expression.clone())
                .then(just(SyntaxKind::COLON))
                .then(expression.clone())
                .group_as(SyntaxKind::MAP_PAIR);

            let map = map_pair
                .repeated_with_trailing_separator(SyntaxKind::COMMA, SyntaxKind::RBRACE)
                .delimited(SyntaxKind::MAP_BRACE, SyntaxKind::RBRACE)
                .group_as(SyntaxKind::MAP);

            let match_clause = self
                .pattern()
                .then(just(SyntaxKind::IF).then(expression.clone()).optional())
                .then(just(SyntaxKind::FAT_ARROW))
                .then(expression.clone())
                .group_as(SyntaxKind::MATCH_CLAUSE);

            // 'match' expr '{' match_clause* '}'
            let match_expression = just(SyntaxKind::MATCH)
                .then(expression.clone())
                .then(
                    match_clause
                        .repeated(SyntaxKind::RBRACE)
                        .delimited(SyntaxKind::LBRACE, SyntaxKind::RBRACE),
                )
                .group_as(SyntaxKind::MATCH);

            let expr_atom = just(SyntaxKind::IDENTIFIER)
                .or(just(SyntaxKind::INTEGER))
                .or(just(SyntaxKind::FRACTIONAL))
                .or(just(SyntaxKind::ATOM))
                .or(let_expression)
                .or(tuple)
                .or(map)
                .or(expression.delimited(SyntaxKind::LPAREN, SyntaxKind::RPAREN))
                .or(match_expression);

            expr_atom.group_as(SyntaxKind::EXPRESSION)
        })
    }

    fn r#type(&self) -> ParserCombinator<'source, Self> {
        ParserCombinator::recursive(|r#type| {
            let map_pair = just(SyntaxKind::IDENTIFIER)
                .then(just(SyntaxKind::COLON))
                .then(r#type.clone())
                .group_as(SyntaxKind::MAP_PAIR);

            let map = map_pair
                .repeated_with_trailing_separator(SyntaxKind::COMMA, SyntaxKind::RBRACE)
                .delimited(SyntaxKind::MAP_BRACE, SyntaxKind::RBRACE)
                .group_as(SyntaxKind::MAP);

            let tuple = r#type
                .clone()
                .group_as(SyntaxKind::TUPLE_ITEM)
                .repeated_with_trailing_separator(SyntaxKind::COMMA, SyntaxKind::RBRACE)
                .delimited(SyntaxKind::LBRACE, SyntaxKind::RBRACE)
                .group_as(SyntaxKind::TUPLE);

            let generic_application = just(SyntaxKind::IDENTIFIER).then(
                r#type
                    .clone()
                    .group_as(SyntaxKind::GENERIC_ARG)
                    .repeated_with_trailing_separator(SyntaxKind::COMMA, SyntaxKind::RBRACKET)
                    .delimited(SyntaxKind::LBRACKET, SyntaxKind::RBRACKET)
                    .group_as(SyntaxKind::GENERIC_ARGS)
                    .optional(),
            );

            let type_atom = generic_application
                .or(just(SyntaxKind::ATOM))
                .or(map)
                .or(tuple);

            let type_intersection = ParserCombinator::recursive(|type_intersection| {
                type_atom
                    .clone()
                    .then(
                        just(SyntaxKind::AMPERSAND)
                            .then(type_intersection)
                            .optional(),
                    )
                    .group_as(SyntaxKind::TYPE_INTERSECTION)
            });

            let type_union = ParserCombinator::recursive(|type_union| {
                type_intersection
                    .clone()
                    .then(just(SyntaxKind::PIPE).then(type_union).optional())
                    .group_as(SyntaxKind::TYPE_UNION)
            });

            type_union
        })
    }

    fn generic_argument_introduction(&self) -> ParserCombinator<'source, Self> {
        self.r#type()
            .repeated_with_trailing_separator(SyntaxKind::COMMA, SyntaxKind::RBRACKET)
            .delimited(SyntaxKind::LBRACKET, SyntaxKind::RBRACKET)
            .group_as(SyntaxKind::GENERIC_INTRODUCTION)
    }

    fn import_definition(&self) -> ParserCombinator<'source, Self> {
        ParserCombinator::when(SyntaxKind::IMPORT, |this: &mut Self| {
            this.expect(SyntaxKind::IMPORT)?;
            this.consume_trivia();
            ParseResult::Ok
        })
        .then(
            ParserCombinator::immediately_just(SyntaxKind::IDENTIFIER)
                .repeated_with_separator(SyntaxKind::SLASH),
        )
        .then(
            just(SyntaxKind::AS)
                .then(just(SyntaxKind::IDENTIFIER))
                .optional(),
        )
        .group_as(SyntaxKind::IMPORT)
    }

    fn trait_definition(&self) -> ParserCombinator<'source, Self> {
        ParserCombinator::when(SyntaxKind::TRAIT, |this: &mut Self| {
            this.expect(SyntaxKind::TRAIT)?;
            this.expect(SyntaxKind::IDENTIFIER)
        })
        .then(self.generic_argument_introduction().optional())
        .then(
            self.type_definition()
                .or(self.function_definition())
                .repeated(SyntaxKind::RBRACE)
                .delimited(SyntaxKind::LBRACE, SyntaxKind::RBRACE),
        )
        .group_as(SyntaxKind::TRAIT)
    }

    fn type_definition(&self) -> ParserCombinator<'source, Self> {
        ParserCombinator::when(SyntaxKind::TYPE, |this: &mut Self| {
            this.expect(SyntaxKind::TYPE)?;
            this.expect(SyntaxKind::IDENTIFIER)
        })
        .then(self.generic_argument_introduction().optional())
        .then(just(SyntaxKind::EQUAL))
        .then(self.r#type())
        .group_as(SyntaxKind::TYPE)
    }

    fn function_definition(&self) -> ParserCombinator<'source, Self> {
        let args = just(SyntaxKind::IDENTIFIER)
            .then(just(SyntaxKind::COLON))
            .then(self.r#type())
            .group_as(SyntaxKind::FN_ARG);

        ParserCombinator::when(SyntaxKind::FN, |this: &mut Self| {
            this.expect(SyntaxKind::FN)?;
            this.expect(SyntaxKind::IDENTIFIER)
        })
        .then(self.generic_argument_introduction().optional())
        .then(
            args.repeated_with_trailing_separator(SyntaxKind::COMMA, SyntaxKind::RPAREN)
                .optional()
                .group_as(SyntaxKind::FN_ARGS)
                .delimited(SyntaxKind::LPAREN, SyntaxKind::RPAREN),
        )
        .then(just(SyntaxKind::ARROW).then(self.r#type()).optional())
        .then(
            self.expression()
                .repeated(SyntaxKind::RBRACE)
                .delimited(SyntaxKind::LBRACE, SyntaxKind::RBRACE)
                .optional(),
        )
        .group_as(SyntaxKind::FN)
    }

    fn definition(&self) -> ParserCombinator<'source, Self> {
        self.import_definition()
            .or(self.trait_definition())
            .or(self.type_definition())
            .or(self.function_definition())
    }
}

/// Create a [[`rowan::GreenNode`]] from
macro_rules! make_green_node {
    // <nothing>
    // meant to allow KIND {}
    (@entries $builder:expr, ) => {

    };

    // KIND: "content"
    (@entries $builder:expr, $kind:ident : $content:expr $(,)?) => {
        $builder.token((SyntaxKind::$kind).into(), $content);
    };

    // KIND: "content", ...
    (@entries $builder:expr, $kind:ident : $content:expr, $($rest:tt)+) => {
        make_green_node!(@entries $builder, $kind: $content);
        make_green_node!(@entries $builder, $($rest)+);
    };

    // KIND { ... }
    (@entries $builder:expr, $kind:ident { $($children:tt)* } $(,)?) => {
        $builder.start_node((SyntaxKind::$kind).into());
        make_green_node!(@entries $builder, $($children)*);
        $builder.finish_node();
    };

    // KIND { ... }, ...
    (@entries $builder:expr, $kind:ident { $($children:tt)* }, $($rest:tt)+) => {
        $builder.start_node((SyntaxKind::$kind).into());
        make_green_node!(@entries $builder, $($children)*);
        $builder.finish_node();
        make_green_node!(@entries $builder, $($rest)+);
    };

    ($kind:ident { $($children:tt)* } $(,)?) => {{
        let mut builder = ::rowan::GreenNodeBuilder::new();
        builder.start_node((SyntaxKind::$kind).into());
        make_green_node!(@entries builder, $($children)*);
        builder.finish_node();
        builder.finish()
    }};
}

#[cfg(test)]
fn to_syntax_node(green_node: GreenNode) -> crate::lang::SyntaxNode {
    crate::lang::SyntaxNode::new_root(green_node)
}

#[allow(clippy::panic)]
#[cfg(test)]
fn compare_green_nodes(left: GreenNode, right: GreenNode) {
    let left_stringified = format!("{:#?}", to_syntax_node(left));
    let right_stringified = format!("{:#?}", to_syntax_node(right));

    similar_asserts::assert_eq!(left_stringified, right_stringified);
}

#[cfg(test)]
fn parse_and_compare(body: &str, parsed_as: GreenNode) {
    let parse = parse(body);

    assert_eq!(parse.errors, Vec::<&str>::new());

    compare_green_nodes(parse.green_node, parsed_as);
}

#[test]
fn parse_import() {
    parse_and_compare(
        "import std/io as io
import std/fs as fs",
        make_green_node! {
            ROOT {
                IMPORT {
                    IMPORT: "import",
                    WHITESPACE: " ",
                    IDENTIFIER: "std",
                    SLASH: "/",
                    IDENTIFIER: "io",
                    WHITESPACE: " ",
                    AS: "as",
                    WHITESPACE: " ",
                    IDENTIFIER: "io",
                },
                WHITESPACE: "\n",
                IMPORT {
                    IMPORT: "import",
                    WHITESPACE: " ",
                    IDENTIFIER: "std",
                    SLASH: "/",
                    IDENTIFIER: "fs",
                    WHITESPACE: " ",
                    AS: "as",
                    WHITESPACE: " ",
                    IDENTIFIER: "fs",
                },
            }
        },
    );
}

#[test]
fn parse_tuples() {
    parse_and_compare(
        "fn tuples() {
    {}
    { :3, }
    { :3, :gwah }
}",
        make_green_node! {
            ROOT {
                FN {
                    FN: "fn",
                    WHITESPACE: " ",
                    IDENTIFIER: "tuples",
                    LPAREN: "(",
                    FN_ARGS {},
                    RPAREN: ")",
                    WHITESPACE: " ",
                    LBRACE: "{",
                    WHITESPACE: "\n    ",
                    EXPRESSION {
                        TUPLE {
                            LBRACE: "{",
                            RBRACE: "}"
                        }
                    },
                    WHITESPACE: "\n    ",
                    EXPRESSION {
                        TUPLE {
                            LBRACE: "{",
                            WHITESPACE: " ",
                            TUPLE_ITEM {
                                EXPRESSION {
                                    ATOM: ":3"
                                }
                            },
                            COMMA: ",",
                            WHITESPACE: " ",
                            RBRACE: "}",
                        }
                    },
                    WHITESPACE: "\n    ",
                    EXPRESSION {
                        TUPLE {
                            LBRACE: "{",
                            WHITESPACE: " ",
                            TUPLE_ITEM {
                                EXPRESSION {
                                    ATOM: ":3"
                                }
                            },
                            COMMA: ",",
                            WHITESPACE: " ",
                            TUPLE_ITEM {
                                EXPRESSION {
                                    ATOM: ":gwah"
                                }
                            },
                            WHITESPACE: " ",
                            RBRACE: "}",
                        }
                    },
                    WHITESPACE: "\n",
                    RBRACE: "}"
                }
            }
        },
    );
}

#[test]
fn parse_types() {
    parse_and_compare(
        "type Status
    = :online
    | :idle
    | :offline

type never = Never

type Option[T]
    = { :some, T }
    | { :none }

type User = #{
    name: String,
    status: Status,
    display_name: Option[String]
}",
        make_green_node! {
            ROOT {
                TYPE {
                    TYPE: "type",
                    WHITESPACE: " ",
                    IDENTIFIER: "Status",
                    WHITESPACE: "\n    ",
                    EQUAL: "=",
                    WHITESPACE: " ",
                    TYPE_UNION {
                        TYPE_INTERSECTION {
                            ATOM: ":online",
                            WHITESPACE: "\n    ",
                        },
                        PIPE: "|",
                        WHITESPACE: " ",
                        TYPE_UNION {
                            TYPE_INTERSECTION {
                                ATOM: ":idle",
                                WHITESPACE: "\n    ",
                            },
                            PIPE: "|",
                            WHITESPACE: " ",
                            TYPE_UNION {
                                TYPE_INTERSECTION {
                                    ATOM: ":offline",
                                    WHITESPACE: "\n\n",
                                }
                            }
                        }
                    }
                },
                TYPE {
                    TYPE: "type",
                    WHITESPACE: " ",
                    IDENTIFIER: "never",
                    WHITESPACE: " ",
                    EQUAL: "=",
                    WHITESPACE: " ",
                    TYPE_UNION {
                        TYPE_INTERSECTION {
                            IDENTIFIER: "Never",
                            WHITESPACE: "\n\n"
                        }
                    }
                },
                TYPE {
                    TYPE: "type",
                    WHITESPACE: " ",
                    IDENTIFIER: "Option",
                    GENERIC_INTRODUCTION {
                        LBRACKET: "[",
                        TYPE_UNION {
                            TYPE_INTERSECTION {
                                IDENTIFIER: "T",
                            }
                        },
                        RBRACKET: "]",
                    },
                    WHITESPACE: "\n    ",
                    EQUAL: "=",
                    WHITESPACE: " ",
                    TYPE_UNION {
                        TYPE_INTERSECTION {
                            TUPLE {
                                LBRACE: "{",
                                WHITESPACE: " ",
                                TUPLE_ITEM {
                                    TYPE_UNION {
                                        TYPE_INTERSECTION {
                                            ATOM: ":some",
                                        }
                                    }
                                },
                                COMMA: ",",
                                WHITESPACE: " ",
                                TUPLE_ITEM {
                                    TYPE_UNION {
                                        TYPE_INTERSECTION {
                                            IDENTIFIER: "T",
                                            WHITESPACE: " ",
                                        }
                                    }
                                },
                                RBRACE: "}",
                            },
                            WHITESPACE: "\n    ",
                        },
                        PIPE: "|",
                        WHITESPACE: " ",
                        TYPE_UNION {
                            TYPE_INTERSECTION {
                                TUPLE {
                                    LBRACE: "{",
                                    WHITESPACE: " ",
                                    TUPLE_ITEM {
                                        TYPE_UNION {
                                            TYPE_INTERSECTION {
                                                ATOM: ":none",
                                                WHITESPACE: " "
                                            }
                                        }
                                    },
                                    RBRACE: "}"
                                },
                                WHITESPACE: "\n\n"
                            }
                        }
                    }
                },
                TYPE {
                    TYPE: "type",
                    WHITESPACE: " ",
                    IDENTIFIER: "User",
                    WHITESPACE: " ",
                    EQUAL: "=",
                    WHITESPACE: " ",
                    TYPE_UNION {
                        TYPE_INTERSECTION {
                            MAP {
                                MAP_BRACE: "#{",
                                WHITESPACE: "\n    ",
                                MAP_PAIR {
                                    IDENTIFIER: "name",
                                    COLON: ":",
                                    WHITESPACE: " ",
                                    TYPE_UNION {
                                        TYPE_INTERSECTION {
                                            IDENTIFIER: "String"
                                        }
                                    }
                                },
                                COMMA: ",",
                                WHITESPACE: "\n    ",
                                MAP_PAIR {
                                    IDENTIFIER: "status",
                                    COLON: ":",
                                    WHITESPACE: " ",
                                    TYPE_UNION {
                                        TYPE_INTERSECTION {
                                            IDENTIFIER: "Status"
                                        }
                                    },
                                },
                                COMMA: ",",
                                WHITESPACE: "\n    ",
                                MAP_PAIR {
                                    IDENTIFIER: "display_name",
                                    COLON: ":",
                                    WHITESPACE: " ",
                                    TYPE_UNION {
                                        TYPE_INTERSECTION {
                                            IDENTIFIER: "Option",
                                            GENERIC_ARGS {
                                                LBRACKET: "[",
                                                GENERIC_ARG {
                                                    TYPE_UNION {
                                                        TYPE_INTERSECTION {
                                                            IDENTIFIER: "String"
                                                        }
                                                    }
                                                },
                                                RBRACKET: "]"
                                            },
                                            WHITESPACE: "\n"
                                        }
                                    }
                                },
                                RBRACE: "}"
                            }
                        }
                    }
                },
            }
        },
    );
}

pub fn parse(text: &str) -> Parse {
    let mut tokens = Lexer::new(text).collect::<Vec<_>>();

    tokens.reverse();

    let parser = Parser {
        tokens,
        builder: GreenNodeBuilder::new(),
        errors: Vec::new(),
    };

    parser.parse()
}
