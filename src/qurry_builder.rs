use std::{iter::Peekable, str::Chars};

use serde::Serialize;

#[derive(Default, Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct TokenizerPosition {
    pub byte_index: usize,
    pub char_index: usize,
}

pub struct Tokenizer<'a> {
    str: &'a str,
    chars: std::iter::Peekable<Chars<'a>>,
    current: TokenizerPosition,
}

impl<'a> Tokenizer<'a> {
    pub fn new(str: &'a str) -> Self {
        Self {
            chars: str.chars().peekable(),
            str,
            current: Default::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
pub enum TokenizerError {
    InvalidChar(char, TokenizerPosition),
    UnclosedString(TokenizerPosition),
    UnfinishedEscape(TokenizerPosition),
    InvalidEscape(char, TokenizerPosition),
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = std::result::Result<TokenFull, TokenizerError>;

    fn next(&mut self) -> Option<Self::Item> {
        enum TokenizerState {
            Default,
            Ident,
            String,
            Escape,
        }
        let mut state = TokenizerState::Default;
        let mut current = self.current;
        let mut last = current;
        let mut string_builder = String::new();
        while let Some(char) = self.chars.peek().copied() {
            match state {
                TokenizerState::Default => {
                    let mut ret: Option<Token> = None;
                    match char {
                        '(' => ret = Some(Token::LPar),
                        ')' => ret = Some(Token::RPar),
                        '|' => ret = Some(Token::Or),
                        '&' => ret = Some(Token::And),
                        '>' => ret = Some(Token::Gt),
                        '<' => ret = Some(Token::Lt),
                        '=' => ret = Some(Token::Eq),
                        '*' => ret = Some(Token::Star),
                        '^' => ret = Some(Token::Carrot),
                        ':' => ret = Some(Token::Colon),
                        ';' => ret = Some(Token::Semicolon),
                        '"' => {
                            state = TokenizerState::String;
                            self.chars.next();
                            current.byte_index += char.len_utf8();
                            current.char_index += 1;
                        }
                        char if char.is_alphabetic() => state = TokenizerState::Ident,
                        char if char.is_whitespace() => {
                            self.chars.next();
                            current.byte_index += char.len_utf8();
                            current.char_index += 1;
                            self.current = current;
                        }
                        bad_char => {
                            let res = Err(TokenizerError::InvalidChar(bad_char, self.current));
                            self.chars.next();
                            current.byte_index += char.len_utf8();
                            current.char_index += 1;
                            self.current = current;
                            return Some(res);
                        }
                    }

                    if ret.is_some() {
                        self.chars.next();
                        current.byte_index += char.len_utf8();
                        current.char_index += 1;
                        let ret = ret.map(|f| Ok(TokenFull::new(f, self.current, current)));
                        self.current = current;
                        return ret;
                    }
                }
                TokenizerState::Ident => {
                    if char.is_alphabetic() {
                        self.chars.next();
                        current.byte_index += char.len_utf8();
                        current.char_index += 1;
                    } else {
                        let token = Token::Ident(
                            self.str[self.current.byte_index..current.byte_index].to_owned(),
                        );
                        let token = TokenFull::new(token, self.current, last);
                        self.current = last;
                        return Some(Ok(token));
                    }
                }
                TokenizerState::String => match char {
                    '"' => {
                        self.chars.next();
                        current.byte_index += char.len_utf8();
                        current.char_index += 1;
                        let res = Token::Value(string_builder);
                        let res = TokenFull::new(res, self.current, current);
                        self.current = current;

                        return Some(Ok(res));
                    }
                    '\\' => {
                        state = TokenizerState::Escape;
                        self.chars.next();
                        current.byte_index += char.len_utf8();
                        current.char_index += 1;
                    }
                    _ => {
                        string_builder.push(char);
                        self.chars.next();
                        current.byte_index += char.len_utf8();
                        current.char_index += 1;
                    }
                },
                TokenizerState::Escape => match char {
                    '\\' => {
                        string_builder.push(char);
                        self.chars.next();
                        current.byte_index += char.len_utf8();
                        current.char_index += 1;
                        state = TokenizerState::String;
                    }
                    '"' => {
                        string_builder.push(char);
                        self.chars.next();
                        current.byte_index += char.len_utf8();
                        current.char_index += 1;
                        state = TokenizerState::String;
                    }
                    char => {
                        self.chars.next();
                        current.byte_index += char.len_utf8();
                        current.char_index += 1;
                        return Some(Err(TokenizerError::InvalidEscape(char, self.current)));
                    }
                },
            }
            last = current;
        }
        match state {
            TokenizerState::Default => None,
            TokenizerState::Ident => {
                let token = Token::Ident(self.str[self.current.byte_index..].to_owned());
                let token = TokenFull::new(token, self.current, last);
                Some(Ok(token))
            }
            TokenizerState::String => Some(Err(TokenizerError::UnclosedString(self.current))),
            TokenizerState::Escape => Some(Err(TokenizerError::UnfinishedEscape(self.current))),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize)]
pub enum Token {
    LPar,
    RPar,
    Or,
    And,
    Lt,
    Gt,
    Colon,
    Semicolon,
    Carrot,
    Star,
    Eq,
    Ident(String),
    Value(String),
}

#[derive(Debug, Clone, Serialize)]
pub struct TokenFull {
    pub data: Token,
    pub start: TokenizerPosition,
    pub end: TokenizerPosition,
}

impl TokenFull {
    pub fn new(data: Token, start: TokenizerPosition, end: TokenizerPosition) -> Self {
        Self { data, start, end }
    }
}

#[test]
fn toknizer_test() {
    let str = "pa\"\\\"\\\\test\"";
    let tokenizer = Tokenizer::new(str);

    for token in tokenizer {
        match token {
            Ok(ok) => {
                if let Token::Ident(ident) = &ok.data {
                    println!("{:#?}", ok);
                    println!("value: {}", ident);
                } else if let Token::Value(value) = &ok.data {
                    println!("{:#?}", ok);
                    println!("value: {}", value);
                } else {
                    println!("{:#?}", ok);
                }
            }
            Err(err) => {
                println!("err: {:#?}", err);
            }
        }
    }
}
pub struct ExpressionParser<'a, 'b, T, E> {
    tokenizer: Peekable<Tokenizer<'a>>,
    visitor: &'b mut dyn Visitor<T, E>,
}

pub trait Visitor<T, E> {
    fn eq(&mut self, ident: String, value: String) -> Result<T, E>;
    fn lt(&mut self, ident: String, value: String) -> Result<T, E>;
    fn gt(&mut self, ident: String, value: String) -> Result<T, E>;
    fn colon(&mut self, ident: String, value: String) -> Result<T, E>;

    fn or(&mut self, ls: T, rs: T) -> Result<T, E>;
    fn and(&mut self, ls: T, rs: T) -> Result<T, E>;
}

#[derive(Debug, Clone, Serialize)]
pub enum ExpressionParserError<T> {
    TokenizerError(TokenizerError),
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
                    expected: Token::Value(String::new()),
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

impl<'a, 'b, T, E> ExpressionParser<'a, 'b, T, E> {
    pub fn new(expression: &'a str, visitor: &'b mut impl Visitor<T, E>) -> Self {
        Self {
            tokenizer: Tokenizer::new(expression).peekable(),
            visitor,
        }
    }
    pub fn parse(&mut self) -> Result<T, ExpressionParserError<E>> {
        self.parse_top()
    }

    fn parse_top(&mut self) -> Result<T, ExpressionParserError<E>> {
        self.parse_3()
    }

    fn parse_3(&mut self) -> Result<T, ExpressionParserError<E>> {
        let mut ls = self.parse_2()?;

        loop {
            if tok_matches!(self.tokenizer.peek(), Token::Or) {
                self.tokenizer.next();
                let rs = self.parse_2()?;
                ls = unwrap_visitor!(self.visitor.or(ls, rs));
            } else {
                return Ok(ls);
            }
        }
    }

    fn parse_2(&mut self) -> Result<T, ExpressionParserError<E>> {
        let mut ls = self.parse_1()?;

        loop {
            if tok_matches!(self.tokenizer.peek(), Token::And) {
                self.tokenizer.next();
                let rs = self.parse_1()?;
                ls = unwrap_visitor!(self.visitor.or(ls, rs));
            } else {
                return Ok(ls);
            }
        }
    }

    fn parse_1(&mut self) -> Result<T, ExpressionParserError<E>> {
        let tok = unwrap_token!(self.tokenizer.next());
        if tok.data == Token::LPar {
            let expr = self.parse_top();
            expect_tok!(unwrap_token!(self.tokenizer.next()), Token::RPar);
            return expr;
        }
        let ident = expect_ident!(tok);

        let operator = unwrap_token!(self.tokenizer.next());

        match operator.data {
            Token::Eq => {
                let value = expect_value!(unwrap_token!(self.tokenizer.next()));
                Ok(unwrap_visitor!(self.visitor.eq(ident, value)))
            }
            Token::Gt => {
                let value = expect_value!(unwrap_token!(self.tokenizer.next()));
                Ok(unwrap_visitor!(self.visitor.gt(ident, value)))
            }
            Token::Lt => {
                let value = expect_value!(unwrap_token!(self.tokenizer.next()));
                Ok(unwrap_visitor!(self.visitor.lt(ident, value)))
            }
            Token::Colon => {
                let value = expect_value!(unwrap_token!(self.tokenizer.next()));
                Ok(unwrap_visitor!(self.visitor.colon(ident, value)))
            }
            _ => Err(ExpressionParserError::UnexpectedTokenReason {
                got: operator,
                expected: stringify!(Token::Eq | Token::Gt | Token::Lt | Token::Colon),
            }),
        }
    }
}

#[test]
fn test_parser() {
    let search = "(hello:\"lol\" | two > \"2\")";
    struct VisitorTest {}
    impl VisitorTest {
        pub fn new() -> Self {
            Self {}
        }
    }
    impl crate::qurry_builder::Visitor<String, ()> for VisitorTest {
        fn eq(&mut self, ident: String, value: String) -> Result<String, ()> {
            Ok(format!("({}={:#?})", ident, value))
        }
        fn lt(&mut self, ident: String, value: String) -> Result<String, ()> {
            Ok(format!("({}<{:#?})", ident, value))
        }
        fn gt(&mut self, ident: String, value: String) -> Result<String, ()> {
            Ok(format!("({}>{:#?})", ident, value))
        }
        fn colon(&mut self, ident: String, value: String) -> Result<String, ()> {
            Ok(format!("({}:{:#?})", ident, value))
        }

        fn or(&mut self, ls: String, rs: String) -> Result<String, ()> {
            Ok(format!("({}|{})", ls, rs))
        }

        fn and(&mut self, ls: String, rs: String) -> Result<String, ()> {
            Ok(format!("({}&{})", ls, rs))
        }
    }
    for token in Tokenizer::new(search) {
        println!("{:#?}", token);
    }
    let mut visitor = VisitorTest::new();
    let mut expr = ExpressionParser::new(search, &mut visitor);
    let res = expr.parse();
    drop(expr);
    println!("{:#?}", res);
}
