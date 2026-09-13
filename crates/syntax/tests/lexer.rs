use ream_syntax::lexer::{Lexer, span::Token};

fn lex_string(body: &'static str) -> Vec<Token<'static>> {
    Lexer::new(body).collect()
}

fn no_tokens_are_errors(tokens: Vec<Token<'static>>) {
    for token in tokens {
        assert!(!token.is_error(), "{token:?} was an error");
    }
}

#[test]
fn lex_empty_string() {
    assert_eq!(lex_string(""), vec![]);
}

#[test]
fn lex_emoji() {
    use ream_syntax::syntax_kind::SyntaxKind::*;

    assert_eq!(
        lex_string("🥴"),
        vec![Token {
            kind: IDENTIFIER,
            contents: "🥴"
        }]
    );
}

#[test]
fn lex_comments() {
    use ream_syntax::syntax_kind::SyntaxKind::*;

    assert_eq!(
        lex_string(
            "// awa
/// gwah
//! wawa
"
        ),
        vec![
            Token {
                kind: COMMENT,
                contents: "// awa\n"
            },
            Token {
                kind: DOC_COMMENT,
                contents: "/// gwah\n"
            },
            Token {
                kind: SUPER_DOC_COMMENT,
                contents: "//! wawa\n"
            }
        ]
    );
}

#[test]
fn lex_unicode_identifier() {
    use ream_syntax::syntax_kind::SyntaxKind::*;

    assert_eq!(
        lex_string("привет"),
        vec![Token {
            kind: IDENTIFIER,
            contents: "привет"
        }]
    );
}

#[test]
fn lex_tuple() {
    use ream_syntax::syntax_kind::SyntaxKind::*;

    assert_eq!(
        lex_string(" { :awa, gwah } "),
        vec![
            Token {
                kind: WHITESPACE,
                contents: " "
            },
            Token {
                kind: LBRACE,
                contents: "{"
            },
            Token {
                kind: WHITESPACE,
                contents: " "
            },
            Token {
                kind: ATOM,
                contents: ":awa"
            },
            Token {
                kind: COMMA,
                contents: ","
            },
            Token {
                kind: WHITESPACE,
                contents: " "
            },
            Token {
                kind: IDENTIFIER,
                contents: "gwah"
            },
            Token {
                kind: WHITESPACE,
                contents: " "
            },
            Token {
                kind: RBRACE,
                contents: "}"
            },
            Token {
                kind: WHITESPACE,
                contents: " "
            }
        ]
    );
}

#[test]
fn lex_function() {
    use ream_syntax::syntax_kind::SyntaxKind::*;

    assert_eq!(
        lex_string("fn awa(a: B) -> C {}"),
        vec![
            Token {
                kind: FN,
                contents: "fn"
            },
            Token {
                kind: WHITESPACE,
                contents: " "
            },
            Token {
                kind: IDENTIFIER,
                contents: "awa"
            },
            Token {
                kind: LPAREN,
                contents: "("
            },
            Token {
                kind: IDENTIFIER,
                contents: "a"
            },
            Token {
                kind: COLON,
                contents: ":"
            },
            Token {
                kind: WHITESPACE,
                contents: " "
            },
            Token {
                kind: IDENTIFIER,
                contents: "B"
            },
            Token {
                kind: RPAREN,
                contents: ")"
            },
            Token {
                kind: WHITESPACE,
                contents: " "
            },
            Token {
                kind: ARROW,
                contents: "->"
            },
            Token {
                kind: WHITESPACE,
                contents: " "
            },
            Token {
                kind: IDENTIFIER,
                contents: "C"
            },
            Token {
                kind: WHITESPACE,
                contents: " "
            },
            Token {
                kind: LBRACE,
                contents: "{"
            },
            Token {
                kind: RBRACE,
                contents: "}"
            }
        ]
    );
}

#[test]
fn lex_program() {
    no_tokens_are_errors(lex_string(
        "
import std/io
import std/string
import std/async as async

type Status
    = :online
    | :idle
    | :offline

type Option[T]
    = { :some, T }
    | { :none }

type User = #{
    name: String,
    status: Status,
    display_name: Option[String]
}

fn has_display_name(user: User) -> bool {
    match user.display_name {
        { :some, _ } => true,
        { :none } => false,
    }
}
",
    ));
}
