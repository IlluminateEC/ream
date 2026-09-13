use rowan::GreenNode;
use rowan::GreenNodeBuilder;

use crate::lexer::Lexer;
use crate::lexer::span::Token;
use crate::parser::combinator::ParseResult;
use crate::parser::combinator::ParserCombinator;
use crate::parser::combinator::ParserState;
use crate::syntax_kind::SyntaxKind;

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
    consumed_token_count: usize,
}

impl<'source> ParserState for Parser<'source> {
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
        self.consume_trivia();
    }

    fn consume(&mut self) {
        self.skip();
    }

    fn expect(&mut self, kind: Self::TokenKind) -> ParseResult {
        self.expect(kind)
    }

    fn expect_immediate(&mut self, kind: Self::TokenKind) -> ParseResult {
        self.expect_immediate(kind)
    }

    fn total_consumed_tokens(&self) -> usize {
        self.consumed_token_count
    }
}

impl<'source> Parser<'source> {
    fn parse(mut self) -> Parse {
        self.builder.start_node(SyntaxKind::ROOT.into());

        // parse top-level statments
        loop {
            match self.definition() {
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
                    self.skip();
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

    /// Advance one token, adding it to the current branch of the tree builder.
    #[allow(clippy::arithmetic_side_effects)]
    fn skip(&mut self) {
        let Some(token) = self.tokens.pop() else {
            return;
        };

        self.builder.token(token.kind.into(), token.contents);
        self.consumed_token_count += 1;
    }

    /// Peek at the first unprocessed token
    fn peek(&self) -> Option<Token<'source>> {
        self.tokens.last().copied()
    }

    fn consume_trivia(&mut self) {
        while let Some(token) = self.peek()
            && token.is_trivia()
        {
            self.skip();
        }
    }

    fn expect_immediate(&mut self, expected_token: SyntaxKind) -> ParseResult {
        if let Some(token) = self.peek()
            && token.kind == expected_token
        {
            self.skip();

            ParseResult::Ok
        } else if self.peek().is_none() {
            ParseResult::Eof
        } else {
            ParseResult::UnexpectedToken
        }
    }

    fn expect(&mut self, expected_token: SyntaxKind) -> ParseResult {
        self.consume_trivia();

        self.expect_immediate(expected_token)
    }

    fn accept_immediate(&mut self, kind: SyntaxKind) -> bool {
        if let Some(token) = self.peek()
            && token.kind == kind
        {
            self.skip();

            true
        } else {
            false
        }
    }

    fn accept(&mut self, kind: SyntaxKind) -> bool {
        self.consume_trivia();

        self.accept_immediate(kind)
    }

    fn parse_as(
        &mut self,
        kind: SyntaxKind,
        parser: &dyn Fn(&mut Self) -> ParseResult,
    ) -> ParseResult {
        self.builder.start_node(kind.into());

        let ret = parser(self);

        self.builder.finish_node();

        ret
    }

    /// runs the predicate when the token is present, but returns Ok if it isn't
    /// Consumes the optional token automatically
    fn optional_immediate(
        &mut self,
        optional_token: SyntaxKind,
        predicate: &dyn Fn(&mut Self) -> ParseResult,
    ) -> Option<ParseResult> {
        if let Some(token) = self.peek()
            && token.is_kind(optional_token)
        {
            self.skip();
            Some(predicate(self))
        } else {
            None
        }
    }

    fn optional(
        &mut self,
        optional_token: SyntaxKind,
        predicate: &dyn Fn(&mut Self) -> ParseResult,
    ) -> Option<ParseResult> {
        self.consume_trivia();

        self.optional_immediate(optional_token, predicate)
    }

    /// Repeats the predicate while the result is okay & Some, breaks on None
    fn repeated(&mut self, predicate: &dyn Fn(&mut Self) -> Option<ParseResult>) -> ParseResult {
        while let Some(result) = predicate(self) {
            result?;
        }

        ParseResult::Ok
    }

    fn repeated_with_separator(
        &mut self,
        separator: SyntaxKind,
        end: SyntaxKind,
        predicate: &dyn Fn(&mut Self) -> ParseResult,
    ) -> ParseResult {
        predicate(self)?;

        self.consume_trivia();
        while let Some(token) = self.peek()
            && token.is_kind(separator)
        {
            self.skip();
            self.consume_trivia();

            if self.accept(end) {
                ParseResult::Ok?;
            }

            predicate(self)?;
            self.consume_trivia();
        }

        self.optional(separator, &|this| {
            this.skip();
            ParseResult::Ok
        })
        .into()
    }

    fn type_internal(&mut self) -> ParseResult {
        self.consume_trivia();

        if let Some(token) = self.peek() {
            match token.kind {
                SyntaxKind::IDENTIFIER => {
                    self.skip();

                    ParseResult::from(self.optional(SyntaxKind::LBRACKET, &|this| {
                        this.repeated_with_separator(
                            SyntaxKind::COMMA,
                            SyntaxKind::RBRACKET,
                            &Self::r#type,
                        )?;
                        this.expect(SyntaxKind::RBRACKET)
                    }))
                }

                SyntaxKind::ATOM => {
                    self.skip();
                    ParseResult::Ok
                }

                // SyntaxKind::HASH => self.parse_as(SyntaxKind::MAP, &|this| {
                //     this.skip();

                //     this.expect_immediate(SyntaxKind::LBRACE)?;

                //     this.repeated_with_separator(SyntaxKind::COMMA, SyntaxKind::RBRACE, &|this| {
                //         this.consume_trivia();
                //         this.parse_as(SyntaxKind::MAP_PAIR, &|this| {
                //             this.expect(SyntaxKind::IDENTIFIER)?;
                //             this.expect(SyntaxKind::COLON)?;
                //             this.r#type()
                //         })
                //     })?;

                //     this.expect(SyntaxKind::RBRACE)
                // }),
                _ => ParseResult::UnexpectedToken,
            }
        } else {
            ParseResult::Eof
        }
    }

    fn r#type(&mut self) -> ParseResult {
        self.type_internal()?;
        self.consume_trivia();

        while let Some(token) = self.peek()
            && (token.is_kind(SyntaxKind::PIPE) || token.is_kind(SyntaxKind::AMPERSAND))
        {
            self.skip();
            self.type_internal()?;
            self.consume_trivia();
        }

        ParseResult::Ok
    }

    fn import(&mut self) -> ParseResult {
        self.expect(SyntaxKind::IMPORT)?;
        self.expect(SyntaxKind::IDENTIFIER)?;

        self.repeated(&|this| {
            this.optional_immediate(SyntaxKind::SLASH, &|this| -> ParseResult {
                this.expect(SyntaxKind::IDENTIFIER)?;
                ParseResult::Ok
            })
        })?;

        self.optional(SyntaxKind::AS, &|this| this.expect(SyntaxKind::IDENTIFIER))
            .into()
    }

    fn type_definition(&mut self) -> ParseResult {
        self.expect(SyntaxKind::TYPE)?;
        self.expect(SyntaxKind::IDENTIFIER)?;

        ParseResult::from(
            self.optional(SyntaxKind::LBRACKET, &Self::generic_parameter_introduction),
        )?;

        self.expect(SyntaxKind::EQUAL)?;
        self.r#type()
    }

    fn pattern(&mut self) -> ParseResult {
        // TODO: the rest

        self.expect(SyntaxKind::IDENTIFIER)
    }

    fn expression(&mut self) -> ParseResult {
        // TODO: the rest

        self.expect(SyntaxKind::IDENTIFIER)
    }

    fn generic_parameter_introduction(&mut self) -> ParseResult {
        self.repeated_with_separator(SyntaxKind::COMMA, SyntaxKind::RBRACKET, &|this| {
            this.expect(SyntaxKind::IDENTIFIER)?;
            this.optional(SyntaxKind::COLON, &Self::r#type).into()
        })?;

        self.expect(SyntaxKind::RBRACKET)
    }

    fn function_definition(&mut self) -> ParseResult {
        self.expect(SyntaxKind::FN)?;
        self.expect(SyntaxKind::IDENTIFIER)?;

        ParseResult::from(
            self.optional(SyntaxKind::LBRACKET, &Self::generic_parameter_introduction),
        )?;

        self.expect(SyntaxKind::LPAREN)?;

        self.repeated_with_separator(SyntaxKind::COMMA, SyntaxKind::RPAREN, &|this| {
            // TODO: potentially allow non-identifier patterns as arguments?
            this.expect(SyntaxKind::IDENTIFIER)?;
            this.expect(SyntaxKind::COLON)?;
            this.r#type()
        })?;

        self.expect(SyntaxKind::RPAREN)?;

        ParseResult::from(self.optional(SyntaxKind::ARROW, &|this| this.r#type()))?;

        // bodies are optional in traits and externs
        self.optional(SyntaxKind::LBRACE, &|this| {
            this.repeated(&|this| Some(this.expression()))?;

            this.expect(SyntaxKind::RBRACE)
        })
        .into()
    }

    fn trait_definition(&mut self) -> ParseResult {
        self.expect(SyntaxKind::TRAIT)?;
        self.expect(SyntaxKind::IDENTIFIER)?;

        ParseResult::from(
            self.optional(SyntaxKind::LBRACKET, &Self::generic_parameter_introduction),
        )?;

        self.expect(SyntaxKind::LBRACE)?;

        // TODO: narrow definition here to exclude traits and imports
        self.repeated(&|this| Some(this.definition()))?;

        self.expect(SyntaxKind::RBRACE)
    }

    fn definition(&mut self) -> ParseResult {
        // self.consume_trivia();

        // if let Some(token) = self.peek() {
        //     match token.kind {
        //         SyntaxKind::IMPORT => self.parse_as(SyntaxKind::IMPORT, &Self::import),
        //         SyntaxKind::TRAIT => self.parse_as(SyntaxKind::TRAIT, &Self::trait_definition),
        //         SyntaxKind::TYPE => self.parse_as(SyntaxKind::TYPE, &Self::type_definition),
        //         SyntaxKind::FN => self.parse_as(SyntaxKind::FN, &Self::function_definition),
        //         _ => ParseResult::UnexpectedToken,
        //     }
        // } else {
        //     ParseResult::Eof
        // }

        let a = self.definition_composed();

        a(self)
    }

    fn expression_composed(&self) -> ParserCombinator<'source, Self> {
        ParserCombinator::just(SyntaxKind::IDENTIFIER)
    }

    fn type_composed(&self) -> ParserCombinator<'source, Self> {
        ParserCombinator::recursive(|r#type| {
            let map_pair = ParserCombinator::just(SyntaxKind::IDENTIFIER)
                .then(ParserCombinator::just(SyntaxKind::COLON))
                .then(r#type.clone())
                .group_as(SyntaxKind::MAP_PAIR);

            let map = map_pair
                .repeated_with_trailing_separator(SyntaxKind::COMMA, SyntaxKind::RBRACE)
                .delimited(SyntaxKind::MAP_BRACE, SyntaxKind::RBRACE)
                .group_as(SyntaxKind::MAP);

            let generic_application = ParserCombinator::just(SyntaxKind::IDENTIFIER).then(
                r#type
                    .clone()
                    .group_as(SyntaxKind::GENERIC_ARG)
                    .repeated_with_trailing_separator(SyntaxKind::COMMA, SyntaxKind::RBRACKET)
                    .delimited(SyntaxKind::LBRACKET, SyntaxKind::RBRACKET)
                    .group_as(SyntaxKind::GENERIC_ARGS)
                    .optional(),
            );

            let type_atom = generic_application
                .or(ParserCombinator::just(SyntaxKind::ATOM))
                .or(map);

            let type_intersection = ParserCombinator::recursive(|type_intersection| {
                type_atom
                    .clone()
                    .then(
                        ParserCombinator::just(SyntaxKind::AMPERSAND)
                            .then(type_intersection)
                            .optional(),
                    )
                    .group_as(SyntaxKind::TYPE_INTERSECTION)
            });

            let type_union = ParserCombinator::recursive(|type_union| {
                type_intersection
                    .clone()
                    .then(
                        ParserCombinator::just(SyntaxKind::PIPE)
                            .then(type_union)
                            .optional(),
                    )
                    .group_as(SyntaxKind::TYPE_UNION)
            });

            type_union
        })
    }

    fn generic_args_composed(&self) -> ParserCombinator<'source, Self> {
        self.r#type_composed()
            .repeated_with_trailing_separator(SyntaxKind::COMMA, SyntaxKind::RBRACKET)
            .delimited(SyntaxKind::LBRACKET, SyntaxKind::RBRACKET)
            .group_as(SyntaxKind::GENERIC_INTRODUCTION)
    }

    fn import_composed(&self) -> ParserCombinator<'source, Self> {
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
            ParserCombinator::just(SyntaxKind::AS)
                .then(ParserCombinator::just(SyntaxKind::IDENTIFIER))
                .optional(),
        )
        .group_as(SyntaxKind::IMPORT)
    }

    fn trait_composed(&self) -> ParserCombinator<'source, Self> {
        ParserCombinator::when(SyntaxKind::TRAIT, |this: &mut Self| {
            this.expect(SyntaxKind::TRAIT)?;
            this.expect(SyntaxKind::IDENTIFIER)
        })
        .then(self.generic_args_composed().optional())
        .then(
            self.type_definition_composed()
                .or(self.function_definition_composed())
                .repeated(SyntaxKind::RBRACE)
                .delimited(SyntaxKind::LBRACE, SyntaxKind::RBRACE),
        )
        .group_as(SyntaxKind::TRAIT)
    }

    fn type_definition_composed(&self) -> ParserCombinator<'source, Self> {
        ParserCombinator::when(SyntaxKind::TYPE, |this: &mut Self| {
            this.expect(SyntaxKind::TYPE)?;
            this.expect(SyntaxKind::IDENTIFIER)
        })
        .then(self.generic_args_composed().optional())
        .then(ParserCombinator::just(SyntaxKind::EQUAL))
        .then(self.type_composed())
        .group_as(SyntaxKind::TYPE)
    }

    fn function_definition_composed(&self) -> ParserCombinator<'source, Self> {
        let args = ParserCombinator::just(SyntaxKind::IDENTIFIER)
            .then(ParserCombinator::just(SyntaxKind::COLON))
            .then(self.type_composed())
            .group_as(SyntaxKind::FN_ARG);

        ParserCombinator::when(SyntaxKind::FN, |this: &mut Self| {
            this.expect(SyntaxKind::FN)?;
            this.expect(SyntaxKind::IDENTIFIER)
        })
        .then(self.generic_args_composed().optional())
        .then(
            args.repeated_with_trailing_separator(SyntaxKind::COMMA, SyntaxKind::RPAREN)
                .group_as(SyntaxKind::FN_ARGS)
                .delimited(SyntaxKind::LPAREN, SyntaxKind::RPAREN),
        )
        .then(
            ParserCombinator::just(SyntaxKind::ARROW)
                .then(self.type_composed())
                .optional(),
        )
        .group_as(SyntaxKind::FN)
    }

    fn definition_composed(&self) -> ParserCombinator<'source, Self> {
        self.import_composed()
            .or(self.trait_composed())
            .or(self.type_definition_composed())
            .or(self.function_definition_composed())
    }
}

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

fn parse(text: &str) -> Parse {
    let mut tokens = Lexer::new(text).collect::<Vec<_>>();

    tokens.reverse();

    let parser = Parser {
        tokens,
        builder: GreenNodeBuilder::new(),
        errors: Vec::new(),
        consumed_token_count: 0,
    };

    parser.parse()
}
