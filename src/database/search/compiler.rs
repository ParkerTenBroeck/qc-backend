use std::iter::Peekable;

use serde::Serialize;
use serde_json::Value;

use super::tokenizer::{Token, TokenErrorFull, TokenFull, Tokenizer};

pub struct ExpressionParser<'a, 'b, T, E> {
    tokenizer: Peekable<Tokenizer<'a>>,
    visitor: &'b mut dyn Visitor<T, E>,
}

pub enum Thing {
    Value(Value),
    Ident(String),
    Path(String),
}

pub trait Visitor<T, E> {
    fn eq(&mut self, ident: String, value: Value) -> Result<T, E>;
    fn lt(&mut self, ident: String, value: Value) -> Result<T, E>;
    fn gt(&mut self, ident: String, value: Value) -> Result<T, E>;
    fn colon(&mut self, ident: String, value: Value) -> Result<T, E>;
    fn between(&mut self, low_value: Value, ident: String, high_value: Value) -> Result<T, E>;

    fn or(&mut self, ls: T, rs: T) -> Result<T, E>;
    fn and(&mut self, ls: T, rs: T) -> Result<T, E>;
    fn not(&mut self, expr: T) -> Result<T, E>;
}

#[derive(Debug, Clone, Serialize, thiserror::Error)]
pub enum ExpressionParserError<T> {
    TokenizerError(TokenErrorFull),
    UnexpectedEndOfExpression,
    UnexpectedKnownToken {
        expected: Token,
        got: TokenFull,
    },
    UnexpectedTokenReason {
        got: TokenFull,
        expected: &'static str,
    },
    VisitorError(T),
    InvalidParsingStack,
}

impl<T: std::fmt::Debug> std::fmt::Display for ExpressionParserError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

macro_rules! unwrap_token {
    ($expr:expr) => {
        match $expr {
            None => return Err(ExpressionParserError::UnexpectedEndOfExpression),
            Some(Err(err)) => return Err(ExpressionParserError::TokenizerError(err.to_owned())),
            Some(Ok(some)) => some,
        }
    };
}

macro_rules! tok_matches {
    ($expr:expr, $pat:pat) => {
        match $expr {
            None => false,
            Some(Err(err)) => return Err(ExpressionParserError::TokenizerError(err.to_owned())),
            Some(Ok(some)) => matches!(some.data, $pat),
        }
    };
}

macro_rules! expect_ident {
    ($expr:expr) => {{
        let token = $expr;
        match token.data {
            Token::Ident(ident) => ident,
            _ => {
                return Err(ExpressionParserError::UnexpectedKnownToken {
                    got: token,
                    expected: Token::Ident(String::new()),
                })
            }
        }
    }};
}

macro_rules! expect_value {
    ($expr:expr) => {{
        let token = $expr;
        match token.data {
            Token::Value(value) => value,
            _ => {
                return Err(ExpressionParserError::UnexpectedKnownToken {
                    got: token,
                    expected: Token::Value(serde_json::Value::Null),
                })
            }
        }
    }};
}

macro_rules! unwrap_visitor {
    ($expr:expr) => {
        match $expr {
            Ok(ok) => ok,
            Err(err) => return Err(ExpressionParserError::VisitorError(err)),
        }
    };
}

macro_rules! expect_tok {
    ($token:expr, $needed:pat) => {{
        let token = $token;
        if matches!(token.data, $needed) {
            token.data
        } else {
            return Err(ExpressionParserError::UnexpectedTokenReason {
                got: token,
                expected: stringify!($needed),
            });
        }
    }};
}

macro_rules! pop_stack {
    ($expr:expr) => {
        match $expr.pop() {
            Some(some) => some,
            None => return Err(ExpressionParserError::InvalidParsingStack),
        }
    };
}

impl<'a, 'b, T, E> ExpressionParser<'a, 'b, T, E> {
    pub fn new(expression: &'a str, visitor: &'b mut impl Visitor<T, E>) -> Self {
        Self {
            tokenizer: Tokenizer::new(expression).peekable(),
            visitor,
        }
    }

    pub fn parse(&mut self) -> Result<T, ExpressionParserError<E>> {
        #[derive(Debug)]
        enum State {
            OrCalled,
            OrReturned1,
            OrReturned2,
            AndCalled,
            AndReturned1,
            AndReturned2,
            BottomCalled,
            BottomReturn1,
            BottomReturn2,
        }
        let mut state_stack = vec![State::OrCalled];
        let mut var_stack = vec![];

        while let Some(state) = state_stack.pop() {
            match state {
                State::OrCalled => {
                    state_stack.push(State::OrReturned1);
                    state_stack.push(State::AndCalled);
                }
                State::OrReturned1 => {
                    if tok_matches!(self.tokenizer.peek(), Token::Or) {
                        self.tokenizer.next();
                        state_stack.push(State::OrReturned2);
                        state_stack.push(State::AndCalled);
                    } else {
                        //return
                    }
                }
                State::OrReturned2 => {
                    let (rs, ls) = (pop_stack!(var_stack), pop_stack!(var_stack));
                    var_stack.push(unwrap_visitor!(self.visitor.or(ls, rs)));
                    state_stack.push(State::OrReturned1);
                }
                State::AndCalled => {
                    state_stack.push(State::AndReturned1);
                    state_stack.push(State::BottomCalled);
                }
                State::AndReturned1 => {
                    if tok_matches!(self.tokenizer.peek(), Token::And) {
                        self.tokenizer.next();
                        state_stack.push(State::AndReturned2);
                        state_stack.push(State::BottomCalled);
                    } else {
                        //return
                    }
                }
                State::AndReturned2 => {
                    let (rs, ls) = (pop_stack!(var_stack), pop_stack!(var_stack));
                    var_stack.push(unwrap_visitor!(self.visitor.and(ls, rs)));
                    state_stack.push(State::AndReturned1);
                }
                State::BottomCalled => {
                    let tok = unwrap_token!(self.tokenizer.next());

                    if tok.data == Token::LPar {
                        state_stack.push(State::BottomReturn1);
                        state_stack.push(State::OrCalled);
                        continue;
                    } else if let Token::Value(low_value) = tok.data {
                        expect_tok!(unwrap_token!(self.tokenizer.next()), Token::Lt);
                        let ident = expect_ident!(unwrap_token!(self.tokenizer.next()));
                        expect_tok!(unwrap_token!(self.tokenizer.next()), Token::Lt);
                        let high_value = expect_value!(unwrap_token!(self.tokenizer.next()));
                        return Ok(unwrap_visitor!(self
                            .visitor
                            .between(low_value, ident, high_value)));
                    } else if tok.data == Token::Bang {
                        state_stack.push(State::BottomReturn2);
                        state_stack.push(State::BottomCalled);
                        continue;
                    }
                    let ident = expect_ident!(tok);

                    let operator = unwrap_token!(self.tokenizer.next());

                    let val = match operator.data {
                        Token::Eq => {
                            let value = expect_value!(unwrap_token!(self.tokenizer.next()));
                            unwrap_visitor!(self.visitor.eq(ident, value))
                        }
                        Token::Gt => {
                            let value = expect_value!(unwrap_token!(self.tokenizer.next()));
                            unwrap_visitor!(self.visitor.gt(ident, value))
                        }
                        Token::Lt => {
                            let value = expect_value!(unwrap_token!(self.tokenizer.next()));
                            unwrap_visitor!(self.visitor.lt(ident, value))
                        }
                        Token::Colon => {
                            let value = expect_value!(unwrap_token!(self.tokenizer.next()));
                            unwrap_visitor!(self.visitor.colon(ident, value))
                        }
                        _ => {
                            return Err(ExpressionParserError::UnexpectedTokenReason {
                                got: operator,
                                expected: stringify!(
                                    Token::Eq | Token::Gt | Token::Lt | Token::Colon
                                ),
                            })
                        }
                    };
                    var_stack.push(val);
                }
                State::BottomReturn1 => {
                    expect_tok!(unwrap_token!(self.tokenizer.next()), Token::RPar);
                }
                State::BottomReturn2 => {
                    let expr = pop_stack!(var_stack);
                    var_stack.push(unwrap_visitor!(self.visitor.not(expr)))
                }
            }
        }

        let ret = pop_stack!(var_stack);
        if !var_stack.is_empty() {
            return Err(ExpressionParserError::InvalidParsingStack);
        }
        Ok(ret)
    }
}

#[test]
fn test_parser() {
    // let search = "!(hello:\"lol\" | two > \"2\" & three < \"3\" | !four = \"4\") & five=\"5\"";
    let mut search = String::new();
    for _ in 0..5000000 {
        search.push('(');
    }
    search.push_str("hellp:\"lol\"");
    for _ in 0..5000000 {
        search.push(')');
    }
    // let search = "!(hellp:\"lol\")";
    struct VisitorTest {}
    impl VisitorTest {
        pub fn new() -> Self {
            Self {}
        }
    }

    impl Visitor<String, ()> for VisitorTest {
        fn eq(&mut self, ident: String, value: Value) -> Result<String, ()> {
            Ok(format!("({}={:#?})", ident, value))
        }
        fn lt(&mut self, ident: String, value: Value) -> Result<String, ()> {
            Ok(format!("({}<{:#?})", ident, value))
        }
        fn gt(&mut self, ident: String, value: Value) -> Result<String, ()> {
            Ok(format!("({}>{:#?})", ident, value))
        }
        fn colon(&mut self, ident: String, value: Value) -> Result<String, ()> {
            Ok(format!("({}:{:#?})", ident, value))
        }
        fn between(
            &mut self,
            low_value: Value,
            ident: String,
            high_value: Value,
        ) -> Result<String, ()> {
            Ok(format!("({:#?}<{}<{:#?})", low_value, ident, high_value))
        }

        fn or(&mut self, ls: String, rs: String) -> Result<String, ()> {
            Ok(format!("({}|{})", ls, rs))
        }

        fn and(&mut self, ls: String, rs: String) -> Result<String, ()> {
            Ok(format!("({}&{})", ls, rs))
        }
        fn not(&mut self, expr: String) -> Result<String, ()> {
            Ok(format!("(!({}))", expr))
        }
    }
    // for token in Tokenizer::new(&search) {
    //     println!("{:#?}", token);
    // }
    let mut visitor = VisitorTest::new();
    let mut expr = ExpressionParser::new(&search, &mut visitor);
    let res = expr.parse();
    drop(expr);
    println!("{:#?}", res);
}
