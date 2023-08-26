use std::str::Chars;

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
    InvalidPath(char, &'static str),
    UnclosedPathIndex(TokenizerPosition),
    UnfinishedPathDot(TokenizerPosition),
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = std::result::Result<TokenFull, TokenizerError>;

    fn next(&mut self) -> Option<Self::Item> {
        enum TokenizerState {
            Default,
            Ident,
            String,
            Escape,

            PathDot,
            PathIndexStart,
            PathIndex,
            PathIdent,
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
                        '!' => ret = Some(Token::Bang),
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
                TokenizerState::Ident => match char {
                    '.' => {
                        self.chars.next();
                        current.byte_index += char.len_utf8();
                        current.char_index += 1;
                        state = TokenizerState::PathDot;
                    }
                    '[' => {
                        self.chars.next();
                        current.byte_index += char.len_utf8();
                        current.char_index += 1;
                        state = TokenizerState::PathIndexStart;
                    }
                    char if char.is_alphanumeric() || char == '_' => {
                        self.chars.next();
                        current.byte_index += char.len_utf8();
                        current.char_index += 1;
                    }
                    _ => {
                        let token = Token::Ident(
                            self.str[self.current.byte_index..current.byte_index].to_owned(),
                        );
                        let token = TokenFull::new(token, self.current, last);
                        self.current = last;
                        return Some(Ok(token));
                    }
                },
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
                TokenizerState::PathDot => {
                    if char.is_alphanumeric() || char == '_' {
                        self.chars.next();
                        current.byte_index += char.len_utf8();
                        current.char_index += 1;
                        state = TokenizerState::PathIdent;
                    } else {
                        return Some(Err(TokenizerError::InvalidPath(
                            char,
                            "Expected identifier after dot operator",
                        )));
                    }
                }
                TokenizerState::PathIndexStart => match char {
                    ']' => {
                        self.chars.next();
                        current.byte_index += char.len_utf8();
                        current.char_index += 1;
                        return Some(Err(TokenizerError::InvalidPath(
                            char,
                            "Cannot close empty path index",
                        )));
                    }
                    '0'..='9' | '#' | '-' => {
                        self.chars.next();
                        current.byte_index += char.len_utf8();
                        current.char_index += 1;
                        state = TokenizerState::PathIndex;
                    }
                    _ => {
                        return Some(Err(TokenizerError::InvalidPath(
                            char,
                            "Unexpected char found in path index",
                        )));
                    }
                },
                TokenizerState::PathIndex => match char {
                    ']' => {
                        self.chars.next();
                        current.byte_index += char.len_utf8();
                        current.char_index += 1;
                        state = TokenizerState::PathIdent;
                    }
                    '0'..='9' | '#' | '-' => {
                        self.chars.next();
                        current.byte_index += char.len_utf8();
                        current.char_index += 1;
                    }
                    _ => {
                        return Some(Err(TokenizerError::InvalidPath(
                            char,
                            "Unexpected char found in path index",
                        )));
                    }
                },
                TokenizerState::PathIdent => match char {
                    '.' => {
                        self.chars.next();
                        current.byte_index += char.len_utf8();
                        current.char_index += 1;
                        state = TokenizerState::PathDot;
                    }
                    '[' => {
                        self.chars.next();
                        current.byte_index += char.len_utf8();
                        current.char_index += 1;
                        state = TokenizerState::PathIndexStart;
                    }
                    char if char.is_alphanumeric() || char == '_' => {
                        self.chars.next();
                        current.byte_index += char.len_utf8();
                        current.char_index += 1;
                    }
                    _ => {
                        let token = Token::Path(
                            self.str[self.current.byte_index..current.byte_index].to_owned(),
                        );
                        let token = TokenFull::new(token, self.current, last);
                        self.current = last;
                        return Some(Ok(token));
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
            TokenizerState::PathDot => Some(Err(TokenizerError::UnfinishedPathDot(self.current))),
            TokenizerState::PathIndexStart | TokenizerState::PathIndex => {
                Some(Err(TokenizerError::UnclosedPathIndex(self.current)))
            }
            TokenizerState::PathIdent => {
                let token = Token::Path(self.str[self.current.byte_index..].to_owned());
                let token = TokenFull::new(token, self.current, last);
                Some(Ok(token))
            }
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
    Bang,
    Eq,
    Ident(String),
    Path(String),
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
    let str = "this is.a test to[0] test[12] this.is[0].really.nice[1]";
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
