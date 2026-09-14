use std::{
    cell::OnceCell,
    ops::{FromResidual, Residual, Try},
    rc::Rc,
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ParseResult {
    Ok,
    Eof,
    UnexpectedToken,
}

impl From<Option<Self>> for ParseResult {
    fn from(value: Option<Self>) -> Self {
        match value {
            Some(result) => result,
            None => Self::Ok,
        }
    }
}

impl Try for ParseResult {
    type Output = ();

    type Residual = Self;

    fn from_output(_output: Self::Output) -> Self {
        Self::Ok
    }

    fn branch(self) -> std::ops::ControlFlow<Self::Residual, Self::Output> {
        match self {
            Self::Ok => std::ops::ControlFlow::Continue(()),
            other => std::ops::ControlFlow::Break(other),
        }
    }
}

impl Residual<()> for ParseResult {
    type TryType = Self;
}

impl FromResidual for ParseResult {
    fn from_residual(residual: <Self as Try>::Residual) -> Self {
        residual
    }
}

pub trait ParserState {
    type TokenKind: PartialEq + Copy + Clone + std::fmt::Debug;

    fn peek_kind(&self) -> Option<Self::TokenKind>;
    fn is_at_eof(&self) -> bool;

    fn start_group(&mut self, kind: Self::TokenKind);
    fn end_group(&mut self);

    fn consume_trivia(&mut self);
    fn consume(&mut self);
    fn expect(&mut self, kind: Self::TokenKind) -> ParseResult;
    fn expect_immediate(&mut self, kind: Self::TokenKind) -> ParseResult;
}

pub struct ParserCombinator<'a, PS>
where
    PS: ParserState,
{
    expected_prefixes: Vec<PS::TokenKind>,
    handler: Rc<dyn Fn(&mut PS) -> ParseResult + 'a>,
}

impl<'a, PS> Clone for ParserCombinator<'a, PS>
where
    PS: ParserState + 'a,
{
    fn clone(&self) -> Self {
        Self {
            expected_prefixes: self.expected_prefixes.clone(),
            handler: self.handler.clone(),
        }
    }
}

impl<'a, PS> ParserCombinator<'a, PS>
where
    PS: ParserState + 'a,
{
    pub fn just(kind: PS::TokenKind) -> Self {
        Self {
            expected_prefixes: vec![kind],
            handler: Rc::new(move |state| state.expect(kind)),
        }
    }

    pub fn immediately_just(kind: PS::TokenKind) -> Self {
        Self {
            expected_prefixes: vec![kind],
            handler: Rc::new(move |state| state.expect_immediate(kind)),
        }
    }

    pub fn when(kind: PS::TokenKind, predicate: impl Fn(&mut PS) -> ParseResult + 'a) -> Self {
        Self {
            expected_prefixes: vec![kind],
            handler: Rc::new(predicate),
        }
    }

    pub fn group_as(self, kind: PS::TokenKind) -> Self {
        Self {
            expected_prefixes: self.expected_prefixes,
            handler: Rc::new(move |state| {
                state.start_group(kind);

                let result = (self.handler)(state);

                state.end_group();

                result
            }),
        }
    }

    pub fn is_applicable(&self, state: &PS) -> bool {
        state
            .peek_kind()
            .is_some_and(|kind| self.expected_prefixes.contains(&kind))
    }

    pub fn or(self, other: Self) -> Self {
        let mut prefixes = self.expected_prefixes.clone();
        prefixes.extend(&other.expected_prefixes);

        Self {
            expected_prefixes: prefixes,
            handler: Rc::new(move |state| {
                state.consume_trivia();

                if self.is_applicable(state) {
                    (self.handler)(state)
                } else if other.is_applicable(state) {
                    (other.handler)(state)
                } else if state.is_at_eof() {
                    ParseResult::Eof
                } else {
                    ParseResult::UnexpectedToken
                }
            }),
        }
    }

    pub fn repeated_with_trailing_separator(
        self,
        separator: PS::TokenKind,
        end: PS::TokenKind,
    ) -> Self {
        Self {
            expected_prefixes: self.expected_prefixes,
            handler: Rc::new(move |state| {
                state.consume_trivia();

                (self.handler)(state)?;

                state.consume_trivia();

                while let Some(kind) = state.peek_kind()
                    && kind == separator
                {
                    state.consume();

                    state.consume_trivia();

                    if state.peek_kind() == Some(end) {
                        break;
                    }

                    state.consume_trivia();

                    (self.handler)(state)?;

                    state.consume_trivia();
                }

                ParseResult::Ok
            }),
        }
    }

    pub fn repeated_with_mandatory_trailing_separator_for_one_item(
        self,
        separator: PS::TokenKind,
        end: PS::TokenKind,
    ) -> Self {
        Self {
            expected_prefixes: self.expected_prefixes,
            handler: Rc::new(move |state| {
                state.consume_trivia();

                if let Some(kind) = state.peek_kind()
                    && kind == end
                {
                    return ParseResult::Ok;
                }

                (self.handler)(state)?;
                state.expect(separator)?;

                state.consume_trivia();

                if let Some(kind) = state.peek_kind()
                    && kind == end
                {
                    return ParseResult::Ok;
                }

                loop {
                    (self.handler)(state)?;

                    state.consume_trivia();

                    match state.peek_kind() {
                        Some(kind) if kind == end => return ParseResult::Ok,
                        None => return ParseResult::Eof,
                        _ => (),
                    }

                    state.expect(separator)?;
                }
            }),
        }
    }

    pub fn repeated_with_separator(self, separator: PS::TokenKind) -> Self {
        Self {
            expected_prefixes: self.expected_prefixes,
            handler: Rc::new(move |state| {
                state.consume_trivia();

                (self.handler)(state)?;

                state.consume_trivia();

                while state.peek_kind() == Some(separator) {
                    state.consume();

                    state.consume_trivia();

                    (self.handler)(state)?;

                    state.consume_trivia();
                }

                ParseResult::Ok
            }),
        }
    }

    pub fn repeated(self, end: PS::TokenKind) -> Self {
        Self {
            expected_prefixes: self.expected_prefixes,
            handler: Rc::new(move |state| {
                state.consume_trivia();

                while let Some(kind) = state.peek_kind()
                    && kind != end
                {
                    (self.handler)(state)?;

                    state.consume_trivia();
                }

                ParseResult::Ok
            }),
        }
    }

    pub fn delimited(self, before: PS::TokenKind, after: PS::TokenKind) -> Self {
        Self {
            expected_prefixes: vec![before],
            handler: Rc::new(move |state| {
                state.expect(before)?;

                (self.handler)(state)?;

                state.expect(after)
            }),
        }
    }

    pub fn then(self, next: Self) -> Self {
        Self {
            expected_prefixes: self.expected_prefixes,
            handler: Rc::new(move |state| {
                (self.handler)(state)?;

                state.consume_trivia();

                (next.handler)(state)
            }),
        }
    }

    pub fn optional(self) -> Self {
        Self {
            expected_prefixes: self.expected_prefixes.clone(),
            handler: Rc::new(move |state| {
                if self.is_applicable(state) {
                    (self.handler)(state)
                } else {
                    ParseResult::Ok
                }
            }),
        }
    }

    pub fn then_expr(self, next: impl Fn(&mut PS) -> ParseResult + 'a) -> Self {
        Self {
            expected_prefixes: self.expected_prefixes,
            handler: Rc::new(move |state| {
                (self.handler)(state)?;
                state.consume_trivia();
                next(state)
            }),
        }
    }

    #[allow(clippy::expect_used)]
    pub fn recursive<F>(builder: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        let cell: Rc<OnceCell<Self>> = Rc::new(OnceCell::new());
        let cell_clone = cell.clone();

        let proxy = Self {
            expected_prefixes: vec![],
            handler: Rc::new(move |state| {
                let inner = cell_clone
                    .get()
                    .expect("Recursive parser invoked before initialization");

                (inner.handler)(state)
            }),
        };

        let inner = builder(proxy);
        let expected_prefixes = inner.expected_prefixes.clone();

        cell.set(inner).expect("Cell initialization failed");

        Self {
            expected_prefixes,
            handler: Rc::new(move |state| {
                let inner = cell
                    .get()
                    .expect("Recursive parser invoked before initialization");

                (inner.handler)(state)
            }),
        }
    }
}

impl<'a, PS> std::fmt::Debug for ParserCombinator<'a, PS>
where
    PS: ParserState + 'a,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParserCombinator")
            .field("expected_prefixes", &self.expected_prefixes)
            .finish()
    }
}

impl<PS: ParserState> std::ops::FnOnce<(&mut PS,)> for ParserCombinator<'_, PS> {
    type Output = ParseResult;

    extern "rust-call" fn call_once(self, args: (&mut PS,)) -> Self::Output {
        (self.handler)(args.0)
    }
}

impl<PS: ParserState> std::ops::FnMut<(&mut PS,)> for ParserCombinator<'_, PS> {
    extern "rust-call" fn call_mut(&mut self, args: (&mut PS,)) -> Self::Output {
        (self.handler)(args.0)
    }
}

impl<PS: ParserState> std::ops::Fn<(&mut PS,)> for ParserCombinator<'_, PS> {
    extern "rust-call" fn call(&self, args: (&mut PS,)) -> Self::Output {
        (self.handler)(args.0)
    }
}
