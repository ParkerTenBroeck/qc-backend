use std::str::{Chars, FromStr};

use serde::Serialize;
use serde_json::{Number, Value};

#[derive(Default, Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct TokenizerPosition {
    pub byte_index: usize,
    pub char_index: usize,
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
    Value(Value),
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


#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
pub struct TokenErrorFull {
    pub err: TokenizerError,
    pub start: TokenizerPosition,
    pub end: TokenizerPosition,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub enum TokenizerError {
    InvalidChar(char),
    UnclosedString,
    UnfinishedEscape,
    InvalidEscape(String),
    InvalidPath(char, &'static str),
    UnclosedPathIndex,
    UnfinishedPathDot,
    UnclosedJson,
    InvalidJson(String),
    InvalidNumber(String),
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = std::result::Result<TokenFull, TokenErrorFull>;

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

            Number,
            NumberMinus,
            NumberLeadingZero,
            NumberDot,
            NumberDotDig,
            NumberEPM,
            NumberED,

            JsonData {
                obj: bool,
                indent: u32,
            },
            JsonStr {
                obj: bool,
                indent: u32,
                escape: bool,
            },
        }

        let mut state = TokenizerState::Default;
        let mut current = self.current;
        let mut last = current;
        let mut string_builder = String::new();

        let mut escape_start = TokenizerPosition::default();

        loop {
            let char = self.chars.peek().copied();
            let mut ret = None;
            let mut consume_char = true;
            let mut update_current = false;
            let next = {
                let mut tmp = current;
                if let Some(char) = char{
                    tmp.byte_index += char.len_utf8();
                    tmp.char_index += 1;
                }
                tmp
            };
            macro_rules! error_inclusive {
                ($expr:expr) => {
                    ret = Some(Err(TokenErrorFull {
                        err: $expr,
                        start: self.current,
                        end: next,
                    }))
                };
            }
            macro_rules! error_current_char {
                ($expr:expr) => {
                    ret = Some(Err(TokenErrorFull {
                        err: $expr,
                        start: last,
                        end: next,
                    }))
                };
            }
            match state {
                TokenizerState::Default => match char {
                    Some('(') => ret = Some(Ok(Token::LPar)),
                    Some(')') => ret = Some(Ok(Token::RPar)),
                    Some('|') => ret = Some(Ok(Token::Or)),
                    Some('&') => ret = Some(Ok(Token::And)),
                    Some('>') => ret = Some(Ok(Token::Gt)),
                    Some('<') => ret = Some(Ok(Token::Lt)),
                    Some('=') => ret = Some(Ok(Token::Eq)),
                    Some('*') => ret = Some(Ok(Token::Star)),
                    Some('^') => ret = Some(Ok(Token::Carrot)),
                    Some(':') => ret = Some(Ok(Token::Colon)),
                    Some('!') => ret = Some(Ok(Token::Bang)),
                    Some(';') => ret = Some(Ok(Token::Semicolon)),
                    Some(char @ ('[' | '{')) => {
                        state = TokenizerState::JsonData {
                            obj: char == '{',
                            indent: 1,
                        }
                    }

                    Some('-') => state = TokenizerState::NumberMinus,
                    Some('0') => state = TokenizerState::NumberLeadingZero,
                    Some('1'..='9') => state = TokenizerState::Number,
                    Some('"') => state = TokenizerState::String,
                    Some(char) if char.is_alphabetic() => state = TokenizerState::Ident,
                    Some(char) if char.is_whitespace() => update_current = true,
                    Some(bad_char) => {
                        error_current_char!(TokenizerError::InvalidChar(bad_char));
                    }
                    None => return None,
                },
                TokenizerState::Ident => match char {
                    Some('.') => {
                        state = TokenizerState::PathDot;
                    }
                    Some('[') => {
                        state = TokenizerState::PathIndexStart;
                    }
                    Some(char) if char.is_alphanumeric() || char == '_' => {}
                    _ => {
                        consume_char = false;
                        let str = &self.str[self.current.byte_index..current.byte_index];
                        let token = if str.eq_ignore_ascii_case("false") {
                            Token::Value(Value::Bool(false))
                        } else if str.eq_ignore_ascii_case("true") {
                            Token::Value(Value::Bool(true))
                        } else if str.eq_ignore_ascii_case("null") {
                            Token::Value(Value::Null)
                        } else {
                            Token::Ident(str.to_owned())
                        };
                        ret = Some(Ok(token));
                    }
                },
                TokenizerState::String => match char {
                    Some('"') => {
                        let mut string = String::new();
                        std::mem::swap(&mut string, &mut string_builder);
                        let res = Token::Value(Value::String(string));
                        ret = Some(Ok(res));
                    }
                    Some('\\') => {
                        escape_start = current;
                        state = TokenizerState::Escape;
                    }
                    Some(char) => {
                        string_builder.push(char);
                    }
                    None => error_inclusive!(TokenizerError::UnclosedString),
                },
                TokenizerState::Escape => match char {
                    Some('\\') => {
                        string_builder.push('\\');
                        state = TokenizerState::String;
                    }
                    Some('"') => {
                        string_builder.push('"');
                        state = TokenizerState::String;
                    }
                    Some('0') => {
                        string_builder.push('\0');
                        state = TokenizerState::String;
                    }
                    Some(_) => {
                        ret = Some(Err(TokenErrorFull{
                            err: TokenizerError::InvalidEscape(self.str[escape_start.byte_index..next.byte_index].to_owned()),
                            start: escape_start,
                            end: next,
                        }));
                    }
                    None => {
                        ret = Some(Err(TokenErrorFull{
                            err: TokenizerError::UnfinishedEscape,
                            start: escape_start,
                            end: next,
                        }));
                    }
                },
                TokenizerState::PathDot => match char {
                    Some(char) if char.is_alphabetic() || char == '_' => {
                        state = TokenizerState::PathIdent;
                    }
                    Some(char) => {
                        consume_char = false;
                        error_inclusive!(TokenizerError::InvalidPath(
                            char,
                            "Expected identifier after dot operator",
                        ));
                    }
                    None => {
                        error_inclusive!(TokenizerError::UnfinishedPathDot);
                    }
                },
                TokenizerState::PathIndexStart => match char {
                    Some(char @ ']') => {
                        error_inclusive!(TokenizerError::InvalidPath(
                            char,
                            "Cannot close empty path index",
                        ));
                    }
                    Some('0'..='9' | '#' | '-') => {
                        state = TokenizerState::PathIndex;
                    }
                    Some(char) => {
                        consume_char = false;
                        error_inclusive!(TokenizerError::InvalidPath(
                            char,
                            "Unexpected char found in path index",
                        ));
                    }
                    None => {
                        error_inclusive!(TokenizerError::UnclosedPathIndex);
                    }
                },
                TokenizerState::PathIndex => match char {
                    Some(']') => {
                        state = TokenizerState::PathIdent;
                    }
                    Some('0'..='9' | '#' | '-') => {}
                    Some(char) => {
                        consume_char = false;
                        error_inclusive!(TokenizerError::InvalidPath(
                            char,
                            "Unexpected char found in path index",
                        ));
                    }
                    None => {
                        error_inclusive!(TokenizerError::UnclosedPathIndex);
                    }
                },
                TokenizerState::PathIdent => match char {
                    Some('.') => {
                        state = TokenizerState::PathDot;
                    }
                    Some('[') => {
                        state = TokenizerState::PathIndexStart;
                    }
                    Some(char) if char.is_alphanumeric() || char == '_' => {}
                    _ => {
                        consume_char = false;
                        let token = Token::Path(
                            self.str[self.current.byte_index..current.byte_index].to_owned(),
                        );
                        ret = Some(Ok(token));
                    }
                },
                TokenizerState::NumberMinus => {
                    if let Some('0') = char{
                        state = TokenizerState::NumberLeadingZero;
                    }else{
                        state = TokenizerState::Number;
                        consume_char = false;
                    }
                },
                TokenizerState::NumberLeadingZero => {
                    match char{
                        Some('1'..='9') => state = TokenizerState::Number,
                        Some('0') => error_inclusive!(TokenizerError::InvalidNumber("Too many leading zeros".into())),
                        Some('e'|'E') => state = TokenizerState::NumberEPM,
                        Some('.') => state = TokenizerState::NumberDot,
                        _ =>{
                            consume_char = false;
                            ret = Some(Ok(Token::Value(Value::Number(0.into()))));
                        }
                    }
                }
                TokenizerState::Number => {
                    match char{
                        Some('0'..='9') => state = TokenizerState::Number,
                        Some('e'|'E') => state = TokenizerState::NumberEPM,
                        Some('.') => state = TokenizerState::NumberDot,
                        _ => {
                            consume_char = false;
                            match Number::from_str(&self.str[self.current.byte_index..current.byte_index]){
                                Ok(ok) => ret = Some(Ok(Token::Value(Value::Number(ok)))),
                                Err(er) => {
                                    error_inclusive!(TokenizerError::InvalidNumber (format!("{:?}", er)));
                                },
                            }
                        }
                    }
                },
                TokenizerState::NumberDot => {
                    match char{
                        Some('0'..='9') => state = TokenizerState::NumberDotDig,
                        _ => {
                            error_inclusive!(TokenizerError::InvalidNumber("Expected 0-9 after .".into()));
                        },
                    }
                }
                TokenizerState::NumberDotDig => {
                    match char{
                        Some('0'..='9') => state = TokenizerState::NumberDotDig,
                        Some('e'|'E') => state = TokenizerState::NumberEPM,
                        _ => {
                            consume_char = false;
                            match Number::from_str(&self.str[self.current.byte_index..current.byte_index]){
                                Ok(ok) => ret = Some(Ok(Token::Value(Value::Number(ok)))),
                                Err(er) => {
                                    error_inclusive!(TokenizerError::InvalidNumber(format!("{:?}", er)));
                                },
                            }
                        }
                    }
                },
                TokenizerState::NumberEPM => match char{
                    Some('+'|'-'|'0'..='9') => state = TokenizerState::NumberED,
                    _ => error_inclusive!(TokenizerError::InvalidNumber("invalid char, expected + or - or 0-9".into())),
                },
                TokenizerState::NumberED => {
                    match char{
                        Some('0'..='9') => state = TokenizerState::NumberED,
                        _ => {
                            consume_char = false;
                            match Number::from_str(&self.str[self.current.byte_index..current.byte_index]){
                                Ok(ok) => ret = Some(Ok(Token::Value(Value::Number(ok)))),
                                Err(er) => {
                                    error_inclusive!(TokenizerError::InvalidNumber(format!("{:?}", er)));
                                },
                            }
                        }
                    }
                },

                TokenizerState::JsonData { obj, indent } => match (obj, char) {
                    (true, Some('}')) | (false, Some(']')) => {
                        let indent = indent - 1;
                        if indent == 0 {
                            let str = &self.str[self.current.byte_index..next.byte_index];
                            match serde_json::from_str(str) {
                                Ok(ok) => {
                                    ret = Some(Ok(Token::Value(ok)));
                                }
                                Err(err) => {
                                    error_inclusive!(TokenizerError::InvalidJson(format!("{:?}", err)));
                                }
                            }
                        } else {
                            state = TokenizerState::JsonData { obj, indent }
                        }
                    }
                    (true, Some('{')) | (false, Some('[')) if obj => {
                        state = TokenizerState::JsonData {
                            obj,
                            indent: indent + 1,
                        }
                    }
                    (_, Some('"')) => {
                        state = TokenizerState::JsonStr {
                            obj,
                            indent,
                            escape: false,
                        }
                    }
                    (_, Some(_)) => {}
                    (_, None) =>  error_inclusive!(TokenizerError::UnclosedJson),
                },
                TokenizerState::JsonStr {
                    obj,
                    indent,
                    escape,
                } => match char {
                    Some('"') if !escape => state = TokenizerState::JsonData { obj, indent },
                    Some('\\') => {
                        state = TokenizerState::JsonStr {
                            obj,
                            indent,
                            escape: true,
                        }
                    }
                    Some(_) => {
                        state = TokenizerState::JsonStr {
                            obj,
                            indent,
                            escape: false,
                        }
                    }
                    None => error_inclusive!(TokenizerError::UnclosedString),
                },
            }

            if consume_char {
                self.chars.next();
                current = next;
            }
            let ret = match ret {
                Some(Ok(ok)) => {
                    if consume_char {
                        Some(Ok(TokenFull {
                            data: ok,
                            start: self.current,
                            end: next,
                        }))
                    } else {
                        Some(Ok(TokenFull {
                            data: ok,
                            start: self.current,
                            end: last,
                        }))
                    }
                }
                Some(Err(err)) => Some(Err(err)),
                None => None,
            };
            if update_current | ret.is_some() {
                if consume_char {
                    self.current = current;
                } else {
                    self.current = last;
                }
            }
            if let Some(ret) = ret {
                return Some(ret);
            }
            last = current;
        }
    }
}


#[test]
fn toknizer_test() {
    let str = r#"{"test": {"wow": 12}, "vals}": "{}"}% & this : "is\" really \\ neat" """" () this is.a test to[0] test[12]  this.is[0].really.nice[1]"#;
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
