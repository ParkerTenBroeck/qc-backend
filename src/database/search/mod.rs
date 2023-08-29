use diesel::sqlite::Sqlite;

use rocket::form::Form;

use rocket::serde::json::Json;

use rocket_sync_db_pools::diesel;
use serde_json::Value;

use crate::database::search::compiler::ExpressionParser;
use crate::time::Time;

pub mod compiler;
pub mod tokenizer;

#[derive(Debug, Serialize, thiserror::Error)]
pub enum VisitorError {
    #[error("Given a null vlaue when expecting only non null values")]
    ExpectedNonNull,
    #[error("Invalid type encountered when using the like operator: type='{0}'")]
    InvalidTypeUsedWithLikeOperator(&'static str),
    #[error("Invalid type encountered when using the eq operator: type='{0}'")]
    InvalidTypeUsedWithEqOperator(&'static str),
    #[error("Invalid type encountered when using the lt operator: type='{0}'")]
    InvalidTypeUsedWithLtOperator(&'static str),
    #[error("Invalid type encountered when using the gt operator: type='{0}'")]
    InvalidTypeUsedWithGtOperator(&'static str),
    #[error("Invalid type encountered when using the between operator: type='{0}'")]
    InvalidTypeUsedWithBetweenOperator(&'static str),
    #[error("Error while trying to parse json data/values: {0}")]
    JsonParsingError(String),
    #[error("Invalid Column Selected: {0}")]
    InvalidColumnSelected(String),
}

use self::compiler::{Visitor, ExpressionParserError};
use self::diesel::prelude::*;
use self::tokenizer::{TokenFull, Tokenizer, TokenErrorFull};

use super::*;

#[get("/tokenize/<str>")]
pub(super) async fn tokenize(str: &str) -> Json<Vec<Result<TokenFull, TokenErrorFull>>> {
    Tokenizer::new(str).collect::<Vec<_>>().into()
}

#[derive(Clone, Debug, Serialize)]
pub enum Infallible{}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type", content="data")]
pub enum Node{
    Eq(String, Value),
    Lt(String, Value),
    Gt(String, Value),
    Between(Value, String, Value),
    Colon(String, Value),
    And(Box<Node>, Box<Node>),
    Or(Box<Node>, Box<Node>),
    Not(Box<Node>),
}

struct CompilerVisitor;
impl Visitor<Node, Infallible> for CompilerVisitor{
    fn eq(&mut self, ident: String, value: Value) -> std::result::Result<Node, Infallible> {
        Ok(Node::Eq(ident, value))
    }

    fn lt(&mut self, ident: String, value: Value) -> std::result::Result<Node, Infallible> {
        Ok(Node::Lt(ident, value))
    }

    fn gt(&mut self, ident: String, value: Value) -> std::result::Result<Node, Infallible> {
        Ok(Node::Gt(ident, value))
    }

    fn colon(&mut self, ident: String, value: Value) -> std::result::Result<Node, Infallible> {
        Ok(Node::Colon(ident, value))
    }

    fn between(&mut self, low_value: Value, ident: String, high_value: Value) -> std::result::Result<Node, Infallible> {
        Ok(Node::Between(low_value, ident, high_value))
    }

    fn or(&mut self, ls: Node, rs: Node) -> std::result::Result<Node, Infallible> {
        Ok(Node::Or(Box::new(ls), Box::new(rs)))
    }

    fn and(&mut self, ls: Node, rs: Node) -> std::result::Result<Node, Infallible> {
        Ok(Node::And(Box::new(ls), Box::new(rs)))
    }

    fn not(&mut self, expr: Node) -> std::result::Result<Node, Infallible> {
        Ok(Node::Not(Box::new(expr)))
    }
}

#[get("/compile/<str>")]
pub(super) async fn compile(str: &str) -> Json<Result<Node, ExpressionParserError<Infallible>>> {
    compiler::ExpressionParser::new(str, &mut CompilerVisitor{}).parse().into()
}

#[get("/get_post/<id>")]
pub(super) async fn get_post(db: Db, id: i32) -> Option<Json<ExistingQCForm>> {
    db.run(move |conn| qc_forms::table.find(id).get_result(conn))
        .await
        .map(Json)
        .ok()
}

macro_rules! dyn_qc_form_column {
    ($column:expr, $ident:ident, $succ:block, $fail:block) => {
        dyn_qc_form_column!($column, $ident, $succ, $succ, $succ, $succ, $succ, $fail)
    };
    ($column:expr, $ident:ident, $succ_text:block, $succ_text_optional:block, $succ_id:block, $succ_date:block, $succ_bool:block, $fail:block) => {
        match $column {
            "id" => {
                let $ident = qc_forms::id;
                $succ_id
            }
            "creation_date" => {
                let $ident = qc_forms::creation_date;
                $succ_date
            }
            "last_updated" => {
                let $ident = qc_forms::last_updated;
                $succ_date
            }
            "finalized" => {
                let $ident = qc_forms::finalized;
                $succ_bool
            }
            "build_location" => {
                let $ident = qc_forms::build_location;
                $succ_text
            }
            "build_type" => {
                let $ident = qc_forms::build_type;
                $succ_text
            }
            "drive_type" => {
                let $ident = qc_forms::drive_type;
                $succ_text
            }
            "item_serial" => {
                let $ident = qc_forms::item_serial;
                $succ_text
            }
            "asm_serial" => {
                let $ident = qc_forms::asm_serial;
                $succ_text_optional
            }
            "oem_serial" => {
                let $ident = qc_forms::oem_serial;
                $succ_text
            }
            "make_model" => {
                let $ident = qc_forms::make_model;
                $succ_text
            }
            "mso_installed" => {
                let $ident = qc_forms::mso_installed;
                $succ_bool
            }
            "operating_system" => {
                let $ident = qc_forms::operating_system;
                $succ_text
            }
            "processor_gen" => {
                let $ident = qc_forms::processor_gen;
                $succ_text
            }
            "processor_type" => {
                let $ident = qc_forms::processor_type;
                $succ_text
            }
            "qc_answers" => {
                let $ident = qc_forms::qc_answers;
                $succ_text
            }
            "qc1_initial" => {
                let $ident = qc_forms::qc1_initial;
                $succ_text
            }
            "qc2_initial" => {
                let $ident = qc_forms::qc2_initial;
                $succ_text_optional
            }
            "ram_size" => {
                let $ident = qc_forms::ram_size;
                $succ_text
            }
            "ram_type" => {
                let $ident = qc_forms::ram_type;
                $succ_text
            }
            "drive_size" => {
                let $ident = qc_forms::drive_size;
                $succ_text
            }
            "sales_order" => {
                let $ident = qc_forms::sales_order;
                $succ_text_optional
            }
            "tech_notes" => {
                let $ident = qc_forms::tech_notes;
                $succ_text
            }
            "metadata" => {
                let $ident = qc_forms::metadata;
                $succ_text_optional
            }
            _ => $fail,
        }
    };
}

type DynExpr =
    Box<dyn BoxableExpression<qc_forms::table, Sqlite, SqlType = diesel::sql_types::Bool>>;

struct VisitorTest {}
impl VisitorTest {
    pub fn new() -> Self {
        Self {}
    }
}

macro_rules! unwrap_visitor_exression {
    ($type:ty, $expr:expr) => {{
        let val: $type = match serde_json::from_str(&$expr) {
            Ok(ok) => ok,
            Err(err) => return Err(VisitorError::JsonParsingError(err.to_string())),
        };
        val
    }};
}

macro_rules! unwrap_visitor_value_exression {
    ($type:ty, $expr:expr) => {{
        let val: $type = match serde_json::from_value(Value::String($expr)) {
            Ok(ok) => ok,
            Err(err) => return Err(VisitorError::JsonParsingError(err.to_string())),
        };
        val
    }};
}

impl compiler::Visitor<DynExpr, VisitorError> for VisitorTest {
    fn eq(&mut self, ident: String, value: Value) -> Result<DynExpr, VisitorError> {
        let value = if value.is_string(){
            match value{
                Value::String(str) => str,
                _ => unreachable!()
            }
        }else{
            serde_json::to_string(&value).map_err(|e|VisitorError::JsonParsingError(format!("{:?}", e)))?
        };
        dyn_qc_form_column!(
            ident.as_str(),
            column,
            { Ok(Box::new(column.eq(value))) },
            {
                if value.is_empty() {
                    Ok(Box::new(column.is_null()))
                } else {
                    Ok(Box::new(
                        column
                            .eq(Some(value))
                            .and(column.is_not_null())
                            .assume_not_null(),
                    ))
                }
            },
            {
                Ok(Box::new(
                    column
                        .eq(unwrap_visitor_exression!(i32, value))
                        .assume_not_null(),
                ))
            },
            {
                Ok(Box::new(
                    column.eq(unwrap_visitor_value_exression!(Time, value)),
                ))
            },
            { Ok(Box::new(column.eq(unwrap_visitor_exression!(bool, value)),)) },
            { Err(VisitorError::InvalidColumnSelected(ident)) }
        )
    }
    fn lt(&mut self, ident: String, value: Value) -> Result<DynExpr, VisitorError> {
        let value = if value.is_string(){
            match value{
                Value::String(str) => str,
                _ => unreachable!()
            }
        }else{
            serde_json::to_string(&value).map_err(|e|VisitorError::JsonParsingError(format!("{:?}", e)))?
        };
        dyn_qc_form_column!(
            ident.as_str(),
            column,
            { Ok(Box::new(column.lt(value))) },
            {
                if value.is_empty() {
                    Err(VisitorError::ExpectedNonNull)
                } else {
                    Ok(Box::new(
                        column
                            .lt(Some(value))
                            .and(column.is_not_null())
                            .assume_not_null(),
                    ))
                }
            },
            {
                Ok(Box::new(
                    column
                        .lt(unwrap_visitor_exression!(i32, value))
                        .assume_not_null(),
                ))
            },
            {
                Ok(Box::new(
                    column.lt(unwrap_visitor_value_exression!(Time, value)),
                ))
            },
            { Ok(Box::new(column.lt(unwrap_visitor_exression!(bool, value)),)) },
            { Err(VisitorError::InvalidColumnSelected(ident)) }
        )
    }
    fn gt(&mut self, ident: String, value: Value) -> Result<DynExpr, VisitorError> {
        let value = if value.is_string(){
            match value{
                Value::String(str) => str,
                _ => unreachable!()
            }
        }else{
            serde_json::to_string(&value).map_err(|e|VisitorError::JsonParsingError(format!("{:?}", e)))?
        };
        dyn_qc_form_column!(
            ident.as_str(),
            column,
            { Ok(Box::new(column.gt(value))) },
            {
                if value.is_empty() {
                    Err(VisitorError::ExpectedNonNull)
                } else {
                    Ok(Box::new(
                        column
                            .gt(Some(value))
                            .and(column.is_not_null())
                            .assume_not_null(),
                    ))
                }
            },
            {
                Ok(Box::new(
                    column
                        .gt(unwrap_visitor_exression!(i32, value))
                        .assume_not_null(),
                ))
            },
            {
                Ok(Box::new(
                    column.gt(unwrap_visitor_value_exression!(Time, value)),
                ))
            },
            { Ok(Box::new(column.gt(unwrap_visitor_exression!(bool, value)),)) },
            { Err(VisitorError::InvalidColumnSelected(ident)) }
        )
    }

    fn between(
        &mut self,
        low_value: Value,
        ident: String,
        high_value: Value,
    ) -> Result<DynExpr, VisitorError> {
        let low_value = if low_value.is_string(){
            match low_value{
                Value::String(str) => str,
                _ => unreachable!()
            }
        }else{
            serde_json::to_string(&low_value).map_err(|e|VisitorError::JsonParsingError(format!("{:?}", e)))?
        };
        let high_value = if high_value.is_string(){
            match high_value{
                Value::String(str) => str,
                _ => unreachable!()
            }
        }else{
            serde_json::to_string(&high_value).map_err(|e|VisitorError::JsonParsingError(format!("{:?}", e)))?
        };
        dyn_qc_form_column!(
            ident.as_str(),
            column,
            { Ok(Box::new(column.between(low_value, high_value))) },
            {
                let low_value = if low_value.is_empty() {
                    None
                } else {
                    Some(low_value)
                };
                let high_value = if high_value.is_empty() {
                    None
                } else {
                    Some(high_value)
                };
                let null = low_value.is_none() | high_value.is_none();
                let expr = Box::new(
                    column
                        .between(low_value, high_value)
                        .and(column.is_not_null())
                        .assume_not_null(),
                );
                if null {
                    Ok(Box::new(expr.or(column.is_null())))
                } else {
                    Ok(expr)
                }
            },
            {
                Ok(Box::new(
                    column
                        .between(
                            unwrap_visitor_exression!(i32, low_value),
                            unwrap_visitor_exression!(i32, high_value),
                        )
                        .assume_not_null(),
                ))
            },
            {
                Ok(Box::new(column.between(
                    unwrap_visitor_value_exression!(Time, low_value),
                    unwrap_visitor_value_exression!(Time, high_value),
                )))
            },
            {
                Ok(Box::new(column.between(
                    unwrap_visitor_exression!(bool, low_value),
                    unwrap_visitor_exression!(bool, high_value),
                )))
            },
            { Err(VisitorError::InvalidColumnSelected(ident)) }
        )
    }

    fn colon(&mut self, ident: String, value: Value) -> Result<DynExpr, VisitorError> {
        // qc_forms::creation_date.sql("")
        // diesel::dsl::sql()
        // diesel_dynamic_schema::table("qc_forms").column("c").sql("");
        // diesel::sql_function!
        let value = if value.is_string(){
            match value{
                Value::String(str) => str,
                _ => unreachable!()
            }
        }else{
            serde_json::to_string(&value).map_err(|e|VisitorError::JsonParsingError(format!("{:?}", e)))?
        };
        dyn_qc_form_column!(
            ident.as_str(),
            _column,
            { Ok(Box::new(_column.like(value))) },
            {
                if value.is_empty() {
                    Err(VisitorError::ExpectedNonNull)
                } else {
                    Ok(Box::new(
                        _column
                            .like(Some(value))
                            .and(_column.is_not_null())
                            .assume_not_null(),
                    ))
                }
            },
            { Err(VisitorError::InvalidTypeUsedWithLikeOperator("Option<i32>")) },
            { Err(VisitorError::InvalidTypeUsedWithLikeOperator("DateTime")) },
            { Err(VisitorError::InvalidTypeUsedWithLikeOperator("bool")) },
            { Err(VisitorError::InvalidColumnSelected(ident)) }
        )
    }

    fn or(&mut self, ls: DynExpr, rs: DynExpr) -> Result<DynExpr, VisitorError> {
        Ok(Box::new(ls.or(rs)))
    }

    fn and(&mut self, ls: DynExpr, rs: DynExpr) -> Result<DynExpr, VisitorError> {
        Ok(Box::new(ls.and(rs)))
    }

    fn not(&mut self, expr: DynExpr) -> std::result::Result<DynExpr, VisitorError> {
        Ok(Box::new(expr.eq(false)))
    }
}

#[derive(FromForm, Debug)]
pub(super) struct SearchForm<'f> {
    limit: Option<i64>,
    offset: Option<i64>,
    search: Option<&'f str>,
    order_table: Option<&'f str>,
    ascending: Option<bool>,
}

#[post("/search", data = "<search>")]
pub(super) async fn search(
    db: Db,
    search: Form<SearchForm<'_>>,
) -> Result<Json<Vec<ExistingQCForm>>> {
    let mut boxed = qc_forms::table.into_boxed();

    let mut visitor = VisitorTest::new();
    'search: {
        if let Some(search) = search.search {
            if search.is_empty() || search.trim().is_empty() {
                break 'search;
            }
            let res = ExpressionParser::new(search, &mut visitor).parse();

            match res {
                Ok(ok) => {
                    boxed = boxed.filter(ok);
                }
                Err(err) => Err(err)?,
            }
        }
    }
    {
        let mut order_table = search.order_table.unwrap_or("id");

        if order_table.trim().is_empty() {
            order_table = "id";
        }

        dyn_qc_form_column!(
            order_table.trim(),
            column,
            {
                if search.ascending.unwrap_or(true) {
                    boxed = boxed.order_by(column.asc())
                } else {
                    boxed = boxed.order_by(column.desc())
                }
            },
            {
                return Err(DataBaseError::InvalidColumn(order_table.to_owned()));
            }
        );
    }

    if let Some(limit) = search.limit {
        if let Some(offset) = search.offset {
            boxed = boxed.limit(limit).offset(offset)
        } else {
            boxed = boxed.limit(limit)
        }
    }

    println!("{}", diesel::debug_query::<Sqlite, _>(&boxed));

    let qc_posts: Vec<ExistingQCForm> = match db.run(move |conn| boxed.load(conn)).await {
        Ok(ok) => ok,
        Err(err) => Err(err)?,
    };

    Ok(qc_posts.into())
}
