use diesel::sql_types::Bool;
use diesel::sqlite::Sqlite;

use rocket::form::Form;

use rocket::serde::json::Json;

use rocket_sync_db_pools::diesel;
use serde_json::Value;

use crate::database::search::compiler::ExpressionParser;

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
    #[error("Invalid Column: {0}")]
    InvalidColumn(String),
}

use self::compiler::{ExpressionParserError, Visitor};
use self::diesel::prelude::*;
use self::tokenizer::{TokenErrorFull, TokenFull, Tokenizer};

use super::*;

#[get("/tokenize/<str>")]
pub(super) async fn tokenize(str: &str) -> Json<Vec<Result<TokenFull, TokenErrorFull>>> {
    Tokenizer::new(str).collect::<Vec<_>>().into()
}

#[derive(Clone, Debug, Serialize)]
pub enum Infallible {}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum Node {
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
impl Visitor<Node, Infallible> for CompilerVisitor {
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

    fn between(
        &mut self,
        low_value: Value,
        ident: String,
        high_value: Value,
    ) -> std::result::Result<Node, Infallible> {
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
    compiler::ExpressionParser::new(str, &mut CompilerVisitor {})
        .parse()
        .into()
}

#[get("/get_post/<id>")]
pub(super) async fn get_post(db: Db, id: i32) -> Option<Json<ExistingQCForm>> {
    db.run(move |conn| qc_forms::table.find(id).get_result(conn))
        .await
        .map(Json)
        .ok()
}

fn to_sql_str(value: &Value) -> String {
    match value {
        Value::Null => "NULL".into(),
        Value::Bool(bool) => (if *bool { "1" } else { "0" }).into(),
        Value::Number(num) => {
            if let Some(uint) = num.as_u64() {
                format!("{}", uint as i64)
            } else if let Some(int) = num.as_i64() {
                format!("{}", int)
            } else if let Some(float) = num.as_f64() {
                format!("{}", float)
            } else {
                num.to_string()
            }
        }
        string @ Value::String(_) => serde_json::to_string(string).unwrap_or("NULL".into()),
        other => serde_json::to_string(other)
            .and_then(|s| serde_json::to_string(&Value::String(s)))
            .unwrap_or("NULL".into()),
    }
}

#[derive(Debug)]
pub enum ColumnType {
    PrimaryId,
    Number,
    Datetime,
    Text,
    Boolean,
    Real,
    Json,
    QcAnswer,
}

pub struct ColumnInfo {
    pub column_name: &'static str,
    pub nullable: bool,
    pub col_type: ColumnType,
}

impl ColumnInfo {
    pub fn new(column_name: &'static str, nullable: bool, col_type: ColumnType) -> Self {
        Self {
            column_name,
            nullable,
            col_type,
        }
    }
}

pub fn verify_column(column: &str) -> Result<ColumnInfo, &str> {
    Ok(match column {
        "id" => ColumnInfo::new("id", true, ColumnType::PrimaryId),
        "creation_date" => ColumnInfo::new("creation_date", false, ColumnType::Datetime),
        "last_updated" => ColumnInfo::new("last_updated", false, ColumnType::Datetime),
        "finalized" => ColumnInfo::new("finalized", false, ColumnType::Boolean),
        "build_location" => ColumnInfo::new("build_location", false, ColumnType::Text),
        "build_type" => ColumnInfo::new("build_type", false, ColumnType::Text),
        "drive_type" => ColumnInfo::new("drive_type", false, ColumnType::Text),
        "item_serial" => ColumnInfo::new("item_serial", false, ColumnType::Text),
        "asm_serial" => ColumnInfo::new("asm_serial", true, ColumnType::Text),
        "oem_serial" => ColumnInfo::new("oem_serial", false, ColumnType::Text),
        "make_model" => ColumnInfo::new("make_model", false, ColumnType::Text),
        "mso_installed" => ColumnInfo::new("mso_installed", false, ColumnType::Boolean),
        "operating_system" => ColumnInfo::new("operating_system", false, ColumnType::Text),
        "processor_gen" => ColumnInfo::new("processor_gen", false, ColumnType::Text),
        "processor_type" => ColumnInfo::new("processor_type", false, ColumnType::Text),
        "qc_answers" => ColumnInfo::new("qc_answers", false, ColumnType::QcAnswer),
        "qc1_initial" => ColumnInfo::new("qc1_initial", false, ColumnType::Text),
        "qc2_initial" => ColumnInfo::new("qc2_initial", true, ColumnType::Text),
        "ram_size" => ColumnInfo::new("ram_size", false, ColumnType::Text),
        "ram_type" => ColumnInfo::new("ram_type", false, ColumnType::Text),
        "drive_size" => ColumnInfo::new("drive_size", false, ColumnType::Text),
        "sales_order" => ColumnInfo::new("sales_order", true, ColumnType::Text),
        "tech_notes" => ColumnInfo::new("tech_notes", false, ColumnType::Text),
        "metadata" => ColumnInfo::new("metadata", false, ColumnType::Json),
        _ => return Err(column),
    })
}

type DynExpr =
    Box<dyn BoxableExpression<qc_forms::table, Sqlite, SqlType = diesel::sql_types::Bool>>;

struct SearchVisitor {}
impl SearchVisitor {
    pub fn new() -> Self {
        Self {}
    }
}

impl compiler::Visitor<DynExpr, VisitorError> for SearchVisitor {
    fn eq(&mut self, ident: String, value: Value) -> Result<DynExpr, VisitorError> {
        use diesel::dsl::*;
        let column = verify_column(&ident).map_err(|c| VisitorError::InvalidColumn(c.into()))?;

        match value {
            Value::Null => Ok(Box::new(sql::<Bool>(column.column_name).sql(" IS NULL"))),
            value => Ok(if column.nullable {
                Box::new(
                    sql::<Bool>("ifnull(")
                        .sql(column.column_name)
                        .sql(" = ")
                        .sql(&to_sql_str(&value))
                        .sql(", FALSE)"),
                )
            } else {
                Box::new(
                    sql::<Bool>(column.column_name)
                        .sql(" = ")
                        .sql(&to_sql_str(&value)),
                )
            }),
        }
    }
    fn lt(&mut self, ident: String, value: Value) -> Result<DynExpr, VisitorError> {
        use diesel::dsl::*;
        let column = verify_column(&ident).map_err(|c| VisitorError::InvalidColumn(c.into()))?;

        match value {
            Value::Null => Ok(Box::new(sql::<Bool>("FALSE"))),
            value => Ok(if column.nullable {
                Box::new(
                    sql::<Bool>("ifnull(")
                        .sql(column.column_name)
                        .sql(" < ")
                        .sql(&to_sql_str(&value))
                        .sql(", FALSE)"),
                )
            } else {
                Box::new(
                    sql::<Bool>(column.column_name)
                        .sql(" < ")
                        .sql(&to_sql_str(&value)),
                )
            }),
        }
    }
    fn gt(&mut self, ident: String, value: Value) -> Result<DynExpr, VisitorError> {
        use diesel::dsl::*;
        let column = verify_column(&ident).map_err(|c| VisitorError::InvalidColumn(c.into()))?;

        match value {
            Value::Null => Ok(Box::new(sql::<Bool>("FALSE"))),
            value => Ok(if column.nullable {
                Box::new(
                    sql::<Bool>("ifnull(")
                        .sql(column.column_name)
                        .sql(" > ")
                        .sql(&to_sql_str(&value))
                        .sql(", FALSE)"),
                )
            } else {
                Box::new(
                    sql::<Bool>(column.column_name)
                        .sql(" > ")
                        .sql(&to_sql_str(&value)),
                )
            }),
        }
    }

    fn between(
        &mut self,
        low_value: Value,
        ident: String,
        high_value: Value,
    ) -> Result<DynExpr, VisitorError> {
        use diesel::dsl::*;
        let column = verify_column(&ident).map_err(|c| VisitorError::InvalidColumn(c.into()))?;

        match (low_value, high_value) {
            (Value::Null, _) | (_, Value::Null) => Ok(Box::new(sql::<Bool>("FALSE"))),
            (low_value, high_value) => Ok(if column.nullable {
                Box::new(
                    sql::<Bool>("ifnull(")
                        .sql(column.column_name)
                        .sql(" BETWEEN ")
                        .sql(&to_sql_str(&low_value))
                        .sql(" AND ")
                        .sql(&to_sql_str(&high_value))
                        .sql(", FALSE)"),
                )
            } else {
                Box::new(
                    sql::<Bool>(column.column_name)
                        .sql(" BETWEEN ")
                        .sql(&to_sql_str(&low_value))
                        .sql(" AND ")
                        .sql(&to_sql_str(&high_value)),
                )
            }),
        }
    }

    fn colon(&mut self, ident: String, value: Value) -> Result<DynExpr, VisitorError> {
        use diesel::dsl::*;
        let column = verify_column(&ident).map_err(|c| VisitorError::InvalidColumn(c.into()))?;

        match value {
            Value::Null => Ok(Box::new(sql::<Bool>("FALSE"))),
            value => Ok(if column.nullable {
                Box::new(
                    sql::<Bool>("ifnull(")
                        .sql(column.column_name)
                        .sql(" LIKE ")
                        .sql(&to_sql_str(&value))
                        .sql(", FALSE)"),
                )
            } else {
                Box::new(
                    sql::<Bool>(column.column_name)
                        .sql(" LIKE ")
                        .sql(&to_sql_str(&value)),
                )
            }),
        }
    }

    fn or(&mut self, ls: DynExpr, rs: DynExpr) -> Result<DynExpr, VisitorError> {
        Ok(Box::new(ls.or(rs)))
    }

    fn and(&mut self, ls: DynExpr, rs: DynExpr) -> Result<DynExpr, VisitorError> {
        Ok(Box::new(ls.and(rs)))
    }

    fn not(&mut self, expr: DynExpr) -> std::result::Result<DynExpr, VisitorError> {
        Ok(Box::new(diesel::dsl::not(expr)))
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

#[post("/search", data = "<search>")]
pub(super) async fn search(
    db: Db,
    search: Form<SearchForm<'_>>,
) -> Result<Json<Vec<ExistingQCForm>>> {
    let mut boxed = qc_forms::table.into_boxed();

    let mut visitor = SearchVisitor::new();
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

        let column = verify_column(order_table.trim())
            .map_err(|c| DataBaseError::InvalidColumn(c.into()))?;
        if search.ascending.unwrap_or(true) {
            dyn_qc_form_column!(
                column.column_name,
                col,
                {
                    boxed = boxed.order_by(col.asc());
                },
                {}
            );
        } else {
            dyn_qc_form_column!(
                column.column_name,
                col,
                {
                    boxed = boxed.order_by(col.desc());
                },
                {}
            );
        }
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
