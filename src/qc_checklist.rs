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

struct QuestionVisitor;

impl<'de> Visitor<'de> for QuestionVisitor {
    type Value = QuestionAnswer;

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "expecting integers 0,1,2,3")
    }

    fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match v {
            0 => Ok(QuestionAnswer::Incomplete),
            1 => Ok(QuestionAnswer::Pass),
            2 => Ok(QuestionAnswer::Fail),
            3 => Ok(QuestionAnswer::NA),
            _ => Err(serde::de::Error::custom(
                "invalid value provided, expected 0,1,2,3",
            )),
        }
    }

    fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_u128(v as u128)
    }
    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_u128(v as u128)
    }
    fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_u128(v as u128)
    }
    fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_u128(v as u128)
    }
    fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_u128(v as u128)
    }
    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_u128(v as u128)
    }
    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_u128(v as u128)
    }
    fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_u128(v as u128)
    }
    fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_u128(v as u128)
    }
}

impl<'de> Deserialize<'de> for QuestionAnswer {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_u8(QuestionVisitor)
    }
}

impl Serialize for QuestionAnswer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u8(match self {
            QuestionAnswer::Incomplete => 0,
            QuestionAnswer::Pass => 1,
            QuestionAnswer::Fail => 2,
            QuestionAnswer::NA => 3,
        })
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, AsExpression, FromSqlRow)]
#[serde(crate = "rocket::serde")]
#[diesel(sql_type = diesel::sql_types::Text)]
pub struct QCChecklist(pub HashMap<String, QuestionAnswer>);

impl QCChecklist {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
}

// impl rocket::form::FromForm for QCChecklist{

// }

impl ToSql<Text, Sqlite> for QCChecklist {
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, Sqlite>,
    ) -> diesel::serialize::Result {
        let val = serde_json::to_string(&self.0)?;
        out.set_value(val);
        Ok(diesel::serialize::IsNull::No)
    }
}

impl FromSql<Text, Sqlite> for QCChecklist {
    fn from_sql(bytes: <Sqlite as Backend>::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        let val = <String as diesel::deserialize::FromSql<Text, Sqlite>>::from_sql(bytes)?;
        Ok(Self(serde_json::from_str(val.as_str())?))
    }
}
