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


#[derive(Debug, Clone, Serialize)]
pub struct ErrorFull {
    pub data: TokenizerError,
    pub start: TokenizerPosition,
    pub end: TokenizerPosition,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub enum TokenizerError {
    InvalidChar(char, TokenizerPosition),
    UnclosedString(TokenizerPosition),
    UnfinishedEscape(TokenizerPosition),
    InvalidEscape(char, TokenizerPosition),
    InvalidPath(char, &'static str),
    UnclosedPathIndex(TokenizerPosition),
    UnfinishedPathDot(TokenizerPosition),
    UnclosedJson(TokenizerPosition),
    InvalidJson {
        msg: String,
        start: TokenizerPosition,
        end: TokenizerPosition,
    },
    InvalidNumber {
        msg: String,
        start: TokenizerPosition,
        end: TokenizerPosition,
    },
}

fn from_str(str: &str) -> (serde_json::Result<Value>, usize) {
    let mut read = serde_json::de::StrRead::new(str);
    let mut de = serde_json::de::Deserializer::new(&mut read);
    let res = serde::de::Deserialize::deserialize(&mut de);
    use serde_json::de::Read;
    let pos = read.byte_offset();
    (res, pos)
}

#[test]
fn bruh() {
    // Number::from_str(s)
    let str = r#"[{}{,}]  ;"#;
    println!("{:#?}", from_str(str));
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
                        let res = Err(TokenizerError::InvalidChar(bad_char, self.current));
                        ret = Some(res);
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
                        let str = &self.str[self.current.byte_index..last.byte_index];
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
                        state = TokenizerState::Escape;
                    }
                    Some(char) => {
                        string_builder.push(char);
                    }
                    None => ret = Some(Err(TokenizerError::UnclosedString(self.current))),
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
                    Some(char) => {
                        ret = Some(Err(TokenizerError::InvalidEscape(char, self.current)));
                    }
                    None => {
                        ret = Some(Err(TokenizerError::UnfinishedEscape(self.current)));
                    }
                },
                TokenizerState::PathDot => match char {
                    Some(char) if char.is_alphabetic() || char == '_' => {
                        state = TokenizerState::PathIdent;
                    }
                    Some(char) => {
                        consume_char = false;
                        ret = Some(Err(TokenizerError::InvalidPath(
                            char,
                            "Expected identifier after dot operator",
                        )));
                    }
                    None => {
                        ret = Some(Err(TokenizerError::UnfinishedPathDot(self.current)));
                    }
                },
                TokenizerState::PathIndexStart => match char {
                    Some(char @ ']') => {
                        ret = Some(Err(TokenizerError::InvalidPath(
                            char,
                            "Cannot close empty path index",
                        )));
                    }
                    Some('0'..='9' | '#' | '-') => {
                        state = TokenizerState::PathIndex;
                    }
                    Some(char) => {
                        consume_char = false;
                        ret = Some(Err(TokenizerError::InvalidPath(
                            char,
                            "Unexpected char found in path index",
                        )));
                    }
                    None => {
                        ret = Some(Err(TokenizerError::UnclosedPathIndex(self.current)));
                    }
                },
                TokenizerState::PathIndex => match char {
                    Some(']') => {
                        state = TokenizerState::PathIdent;
                    }
                    Some('0'..='9' | '#' | '-') => {}
                    Some(char) => {
                        consume_char = false;
                        ret = Some(Err(TokenizerError::InvalidPath(
                            char,
                            "Unexpected char found in path index",
                        )));
                    }
                    None => {
                        ret = Some(Err(TokenizerError::UnclosedPathIndex(self.current)));
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
                            self.str[self.current.byte_index..last.byte_index].to_owned(),
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
                        Some('0') => ret = Some(Err(TokenizerError::InvalidNumber { msg: "Too many leading zeros".into(), start: self.current, end: last })),
                        Some('e'|'E') => state = TokenizerState::NumberEPM,
                        Some('.') => state = TokenizerState::NumberDotDig,
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
                        Some('.') => state = TokenizerState::NumberDotDig,
                        _ => {
                            consume_char = false;
                            match Number::from_str(&self.str[self.current.byte_index..last.byte_index]){
                                Ok(ok) => ret = Some(Ok(Token::Value(Value::Number(ok)))),
                                Err(er) => {
                                    ret = Some(Err(TokenizerError::InvalidNumber { msg: format!("{:?}", er), start: self.current, end: last }));
                                },
                            }
                        }
                    }
                },
                TokenizerState::NumberDot => {
                    match char{
                        Some('0'..='9') => state = TokenizerState::NumberDotDig,
                        _ => {
                            ret = Some(Err(TokenizerError::InvalidNumber { msg: "Expected 0-9 after .".into(), start: self.current, end: current }));
                        },
                    }
                }
                TokenizerState::NumberDotDig => {
                    match char{
                        Some('0'..='9') => state = TokenizerState::NumberDotDig,
                        Some('e'|'E') => state = TokenizerState::NumberEPM,
                        _ => {
                            consume_char = false;
                            match Number::from_str(&self.str[self.current.byte_index..last.byte_index]){
                                Ok(ok) => ret = Some(Ok(Token::Value(Value::Number(ok)))),
                                Err(er) => {
                                    ret = Some(Err(TokenizerError::InvalidNumber { msg: format!("{:?}", er), start: self.current, end: last }));
                                },
                            }
                        }
                    }
                },
                TokenizerState::NumberEPM => match char{
                    Some('+'|'-'|'0'..='9') => state = TokenizerState::NumberED,
                    _ => ret = Some(Err(TokenizerError::InvalidNumber { msg: "invalid char, expected + or - or 0-9".into(), start: self.current, end: last })),
                },
                TokenizerState::NumberED => {
                    match char{
                        Some('0'..='9') => state = TokenizerState::NumberED,
                        _ => {
                            consume_char = false;
                            match Number::from_str(&self.str[self.current.byte_index..last.byte_index]){
                                Ok(ok) => ret = Some(Ok(Token::Value(Value::Number(ok)))),
                                Err(er) => {
                                    ret = Some(Err(TokenizerError::InvalidNumber { msg: format!("{:?}", er), start: self.current, end: last }));
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
                                    ret = Some(Err(TokenizerError::InvalidJson {
                                        msg: format!("{:?}", err),
                                        start: self.current,
                                        end: current,
                                    }));
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
                    (_, None) => ret = Some(Err(TokenizerError::UnclosedJson(self.current))),
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
                    None => ret = Some(Err(TokenizerError::UnclosedString(self.current))),
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
                            end: current,
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
