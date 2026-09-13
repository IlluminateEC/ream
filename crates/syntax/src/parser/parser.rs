use rowan::GreenNode;
use rowan::GreenNodeBuilder;

use crate::lexer::Lexer;
use crate::lexer::span::Token;
use crate::parser::combinator::ParseResult;
use crate::parser::combinator::ParserCombinator;
use crate::parser::combinator::ParserState;
use crate::syntax_kind::SyntaxKind;
use crate::syntax_kind::SyntaxKind::IDENTIFIER;

#[derive(Debug, Clone)]
struct Parse {
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
                .repeated_with_trailing_separator(SyntaxKind::COMMA, SyntaxKind::RBRACE)
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

#[cfg(test)]
fn indentation(depth: usize) -> String {
    "  ".repeat(depth)
}

#[cfg(test)]
#[allow(clippy::arithmetic_side_effects)]
fn print_tree_impl(node: &rowan::GreenNodeData, depth: usize) {
    println!("{}{:?}", indentation(depth), SyntaxKind::from(node.kind()));

    for child in node.children() {
        match child {
            rowan::NodeOrToken::Node(node) => print_tree_impl(&node, depth + 1),
            rowan::NodeOrToken::Token(token) => {
                println!(
                    "{}{:?}: {:?}",
                    indentation(depth + 1),
                    SyntaxKind::from(token.kind()),
                    token.text()
                );
            }
        }
    }
}

#[cfg(test)]
fn print_tree(node: &rowan::GreenNode) {
    print_tree_impl(node, 0);
}

#[test]
fn parse_import() {
    let parse = parse(
        "import std/io as io
import std/fs as fs",
    );

    print_tree(&parse.green_node);

    assert_eq!(parse.errors, Vec::<&str>::new());

    assert_eq!(
        parse.green_node,
        GreenNode::new(SyntaxKind::ROOT.into(), vec![])
    );
}

#[test]
fn parse_more() {
    let parse = parse(
        "import std/io
import std/string
import std/async as async


type Status
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
}

fn awa() {
    let gwah: { :3 } = { :3, }
}
",
    );
    print_tree(&parse.green_node);

    assert_eq!(parse.errors, Vec::<&str>::new());

    assert_eq!(
        parse.green_node,
        GreenNode::new(SyntaxKind::ROOT.into(), vec![])
    );
}

fn parse(text: &str) -> Parse {
    let mut tokens = Lexer::new(text).collect::<Vec<_>>();

    tokens.reverse();

    let parser = Parser {
        tokens,
        builder: GreenNodeBuilder::new(),
        errors: Vec::new(),
    };

    parser.parse()
}
