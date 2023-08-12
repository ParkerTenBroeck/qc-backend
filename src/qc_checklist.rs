use std::collections::HashMap;

use ::serde::{Deserialize, Serialize};
use diesel::{
    backend::Backend,
    deserialize::{FromSql, FromSqlRow},
    expression::AsExpression,
    serialize::ToSql,
    sql_types::Text,
    sqlite::Sqlite,
};
use serde::de::Visitor;

#[derive(Debug, Hash, Clone, Copy, PartialEq, Eq)]
pub enum QuestionAnswer {
    Incomplete,
    Pass,
    Fail,
    NA,
}

#[derive(Debug, Hash, Clone, Copy, PartialEq, Eq)]
pub struct QuestionAnswers(pub [QuestionAnswer; 2]);

impl QuestionAnswers {}

impl Serialize for QuestionAnswers {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let bytes = [self.0[0].as_char() as u8, self.0[1].as_char() as u8];
        let str = std::str::from_utf8(&bytes).unwrap();
        serializer.serialize_str(str)
    }
}

struct QuestionsVisitor;

impl<'de> Visitor<'de> for QuestionsVisitor {
    type Value = QuestionAnswers;

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "expecting string like [pfniPFNI]{{2}}")
    }

    fn visit_str<E>(self, val: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let b = val.as_bytes();
        if b.len() != 2 {
            return Err(serde::de::Error::custom("Invalid answer length"));
        }
        let qca1 = b[0] as char;
        let qca2 = b[1] as char;
        Ok(QuestionAnswers([
            qca1.try_into()
                .map_err(|e| serde::de::Error::custom(format!("{}", e)))?,
            qca2.try_into()
                .map_err(|e| serde::de::Error::custom(format!("{}", e)))?,
        ]))
    }
}

impl<'de> Deserialize<'de> for QuestionAnswers {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(QuestionsVisitor)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuestionAnswerError {
    InvalidChar(char),
}
impl std::fmt::Display for QuestionAnswerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl std::error::Error for QuestionAnswerError {}

impl TryInto<QuestionAnswer> for char {
    type Error = QuestionAnswerError;

    fn try_into(self) -> Result<QuestionAnswer, Self::Error> {
        match self {
            'i' | 'I' => Ok(QuestionAnswer::Incomplete),
            'p' | 'P' => Ok(QuestionAnswer::Pass),
            'f' | 'F' => Ok(QuestionAnswer::Fail),
            'n' | 'N' => Ok(QuestionAnswer::NA),
            _ => Err(Self::Error::InvalidChar(self)),
        }
    }
}

impl QuestionAnswer {
    pub fn as_char(&self) -> char {
        match self {
            QuestionAnswer::Incomplete => 'i',
            QuestionAnswer::Pass => 'p',
            QuestionAnswer::Fail => 'f',
            QuestionAnswer::NA => 'n',
        }
    }
}

struct QuestionVisitor;

impl<'de> Visitor<'de> for QuestionVisitor {
    type Value = QuestionAnswer;

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "expecting characters p,f,n,i or P,F,N,I")
    }

    fn visit_char<E>(self, v: char) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        v.try_into()
            .map_err(|e| serde::de::Error::custom(format!("{}", e)))
    }
}

impl<'de> Deserialize<'de> for QuestionAnswer {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_char(QuestionVisitor)
    }
}

impl Serialize for QuestionAnswer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_char(self.as_char())
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, AsExpression, FromSqlRow)]
#[serde(crate = "rocket::serde")]
#[diesel(sql_type = diesel::sql_types::Text)]
pub struct QCChecklist(pub HashMap<String, QuestionAnswers>);

impl QCChecklist {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
}

impl ToSql<Text, Sqlite> for QCChecklist {
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, Sqlite>,
    ) -> diesel::serialize::Result {
        let mut string = String::new();
        for (key, val) in &self.0 {
            string.push_str(key);
            string.push(':');
            string.push(val.0[0].as_char());
            string.push(val.0[1].as_char());
            string.push(',');
        }
        out.set_value(string);
        Ok(diesel::serialize::IsNull::No)
    }
}

impl FromSql<Text, Sqlite> for QCChecklist {
    fn from_sql(bytes: <Sqlite as Backend>::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        let val = <String as diesel::deserialize::FromSql<Text, Sqlite>>::from_sql(bytes)?;

        let inner = val
            .split(',')
            //this give this some werid syntax like ,,,, being valid but it allows for trailing commans so mid
            .filter(|v| !v.is_empty())
            .map(|v| v.split_once(':').ok_or_else(|| "Invalid entry".into()))
            .map(|r| match r {
                Ok((key, val)) => Ok((key.to_owned(), {
                    let b = val.as_bytes();
                    if b.len() != 2 {
                        return Err("Invalid answer length".into());
                    }
                    let qca1 = b[0] as char;
                    let qca2 = b[1] as char;
                    QuestionAnswers([qca1.try_into()?, qca2.try_into()?])
                })),
                Err(e) => Err(e),
            })
            .collect::<diesel::deserialize::Result<_>>()?;

        Ok(Self(inner))
    }
}
