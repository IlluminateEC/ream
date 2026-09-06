use ream_syntax::lexer::Lexer;

#[test]
fn lex_tuple() {
    let lexer = Lexer::new(" { :awa, gwah } ");

    for token in lexer {
        println!("{token:?}");
    }

    panic!();
}
