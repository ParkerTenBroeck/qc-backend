use crate::admin_pwd::Admin;
use crate::json_text::JsonText;
use diesel::result::DatabaseErrorInformation;
use diesel::sqlite::Sqlite;
use rocket::fairing::AdHoc;
use rocket::form::Form;
use rocket::http::hyper::Response;
use rocket::response::Responder;
use rocket::response::status::Accepted;
use rocket::response::{status::Created, Debug};
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::{Build, Rocket};

use rocket_sync_db_pools::diesel;
use serde_json::Value;

use crate::qc_checklist::QCChecklist;
use crate::qurry_builder::{ExpressionParser, ExpressionParserError};
use crate::time::Time;

use self::diesel::prelude::*;

use crate::schema::*;

#[database("diesel")]
pub struct Db(diesel::SqliteConnection);

impl Db {
    pub async fn get_form(&self, id: i32) -> Result<ExistingQCForm> {
        let form: ExistingQCForm = self
            .run(move |conn| qc_forms::table.filter(qc_forms::id.eq(id)).first(conn))
            .await?;
        Ok(form)
    }
}

#[derive(thiserror::Error, Debug, Serialize)]
pub enum DataBaseError{
    #[error("Tried to update finalized form")]
    UpdatedFinalized,
    #[error("{0}")]
    #[serde(serialize_with = "nothing")]
    DbError(#[from] diesel::result::Error),
    #[error("A form with the provided OEM serial alreadt exists")]
    ExistingOemSerial,
    #[error("A form with the provided ASM serial alreadt exists")]
    ExistingAsmSerial,
    #[error("A form with the provided Item serial alreadt exists")]
    ExistingItemSerial,
    #[error("An error occured when parsing database search query: {0}")]
    DataBaseSearchError(#[from] ExpressionParserError<VisitorError>),
    #[error("Invalid column specified '{0:?}'")]
    InvalidColumn(String),
}
fn nothing<T, S>(_: &T,s: S) -> Result<S::Ok, S::Error> where S: serde::Serializer{
    s.serialize_str("")
}



impl<'r> Responder<'r, 'static> for DataBaseError{
    fn respond_to(self, _: &'r rocket::Request<'_>) -> std::result::Result<rocket::Response<'static>, rocket::http::Status> { 
        use std::io::Cursor;
        use rocket::response::Response;

        Response::build()
            .header(rocket::http::ContentType::Plain)
            .status(rocket::http::Status::BadRequest)
            .streamed_body(Cursor::new(serde_json::to_vec(&self).unwrap_or(Vec::new())))
            .ok()
     }

}

pub type Result<T, E = DataBaseError> = std::result::Result<T, E>;

fn time_default() -> Time {
    Time(time::OffsetDateTime::now_utc())
}



#[derive(Debug, Clone, Deserialize, Serialize, Queryable, Insertable, AsChangeset)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = crate::schema::qc_forms)]
#[diesel(treat_none_as_null = true)]
pub struct NewQCForm {
    #[serde(skip_deserializing)]
    #[serde(default)]
    pub finalized: bool,
    // #[serde(skip_deserializing)]
    // #[serde(default = "time_default")]
    pub creation_date: Time,
    // #[serde(skip_deserializing)]
    // #[serde(default = "time_default")]
    pub last_updated: Time,
    pub build_location: String,
    pub build_type: String,
    pub drive_type: String,
    // example SHIL-0023746
    pub item_serial: String,
    // example CFS-SL300F-001220
    pub asm_serial: Option<String>,
    // the serial listed in the bios of the device
    pub oem_serial: String,
    pub make_model: String,
    pub mso_installed: bool,
    pub operating_system: String,
    pub processor_gen: String,
    pub processor_type: String,
    pub qc_answers: QCChecklist,
    pub qc1_initial: String,
    pub qc2_initial: Option<String>,

    pub ram_size: String,
    pub ram_type: String,

    pub sales_order: Option<String>,
    pub drive_size: String,
    pub tech_notes: String,

    pub metadata: Option<JsonText>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Queryable, Insertable, AsChangeset)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = crate::schema::qc_forms)]
#[diesel(treat_none_as_null = true)]
pub struct ExistingQCForm {
    #[serde(skip_deserializing)]
    pub id: i32,
    #[serde(skip_deserializing)]
    pub finalized: bool,
    #[serde(default = "time_default")]
    pub creation_date: Time,
    #[serde(default = "time_default")]
    pub last_updated: Time,
    pub build_location: String,
    pub build_type: String,
    pub drive_type: String,
    // example SHIL-0023746
    pub item_serial: String,
    // example CFS-SL300F-001220
    pub asm_serial: Option<String>,
    // the serial listed in the bios of the device
    pub oem_serial: String,
    pub make_model: String,
    pub mso_installed: bool,
    pub operating_system: String,
    pub processor_gen: String,
    pub processor_type: String,
    pub qc_answers: QCChecklist,
    pub qc1_initial: String,
    pub qc2_initial: Option<String>,

    pub ram_size: String,
    pub ram_type: String,

    pub sales_order: Option<String>,
    pub drive_size: String,
    pub tech_notes: String,

    pub metadata: Option<JsonText>,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, AsChangeset)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = crate::schema::qc_forms)]
#[serde(default)]
pub struct QCFormUpdate {
    #[serde(skip_deserializing)]
    pub last_updated: Option<Time>,
    pub build_location: Option<String>,
    pub build_type: Option<String>,
    pub drive_type: Option<String>,
    pub item_serial: Option<String>,
    #[serde(deserialize_with = "deserialize_optional_field")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asm_serial: Option<Option<String>>,
    pub oem_serial: Option<String>,
    pub make_model: Option<String>,
    pub mso_installed: Option<bool>,
    pub operating_system: Option<String>,
    pub processor_gen: Option<String>,
    pub processor_type: Option<String>,
    pub qc_answers: Option<QCChecklist>,
    pub qc1_initial: Option<String>,
    #[serde(deserialize_with = "deserialize_optional_field")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qc2_initial: Option<Option<String>>,

    pub ram_size: Option<String>,
    pub ram_type: Option<String>,

    #[serde(deserialize_with = "deserialize_optional_field")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sales_order: Option<Option<String>>,
    pub drive_size: Option<String>,
    pub tech_notes: Option<String>,

    #[serde(deserialize_with = "deserialize_optional_field")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Option<JsonText>>,
}

fn deserialize_optional_field<'de, T, D>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    Ok(Some(Option::deserialize(deserializer)?))
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

// type DynTable = diesel_dynamic_schema::Table<String>;

type DynExpr =
    Box<dyn BoxableExpression<qc_forms::table, Sqlite, SqlType = diesel::sql_types::Bool>>;

#[derive(Debug, Serialize, thiserror::Error)]
pub enum VisitorError{
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
    InvalidColumnSelected(String)
}

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

impl crate::qurry_builder::Visitor<DynExpr, VisitorError> for VisitorTest {
    fn eq(&mut self, ident: String, value: String) -> Result<DynExpr, VisitorError> {
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
            {
                Err(VisitorError::InvalidColumnSelected(ident))
            }
        )
    }
    fn lt(&mut self, ident: String, value: String) -> Result<DynExpr, VisitorError> {
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
            {
                Err(VisitorError::InvalidColumnSelected(ident))
            }
        )
    }
    fn gt(&mut self, ident: String, value: String) -> Result<DynExpr, VisitorError> {
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
            {
                Err(VisitorError::InvalidColumnSelected(ident))
            }
        )
    }

    fn between(
        &mut self,
        low_value: String,
        ident: String,
        high_value: String,
    ) -> Result<DynExpr, VisitorError> {
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
            {
                Err(VisitorError::InvalidColumnSelected(ident))
            }
        )
    }

    fn colon(&mut self, ident: String, value: String) -> Result<DynExpr, VisitorError> {
        // qc_forms::creation_date.sql("")
        // diesel::dsl::sql()
        // diesel_dynamic_schema::table("qc_forms").column("c").sql("");
        // diesel::sql_function!
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
            {
                Err(VisitorError::InvalidTypeUsedWithLikeOperator("Option<i32>"))
            },
            {
                Err(VisitorError::InvalidTypeUsedWithLikeOperator("DateTime"))
            },
            {
                Err(VisitorError::InvalidTypeUsedWithLikeOperator("bool"))
            },
            {
                Err(VisitorError::InvalidColumnSelected(ident))
            }
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
struct SearchForm<'f> {
    limit: Option<i64>,
    offset: Option<i64>,
    search: Option<&'f str>,
    order_table: Option<&'f str>,
    ascending: Option<bool>,
}

#[post("/search", data = "<search>")]
async fn search(
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
                Err(err) => {
                    Err(err)?
                }
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

#[get("/get_post/<id>")]
async fn get_post(db: Db, id: i32) -> Option<Json<ExistingQCForm>> {
    db.run(move |conn| qc_forms::table.find(id).get_result(conn))
        .await
        .map(Json)
        .ok()
}

#[delete("/delete_post/<id>")]
async fn delete_post(db: Db, id: i32, _admin: Admin) -> Result<()>{
    db.run(move |conn| {
        diesel::delete(qc_forms::table.find(id)).execute(conn)?;
        Ok(())
    }).await
}

#[post("/finalize_post/<id>")]
async fn finalize_post(db: Db, id: i32) -> Result<Json<ExistingQCForm>>{
    db.run(move |conn| {
        diesel::update(qc_forms::table.find(id)).set(qc_forms::finalized.eq(true)).execute(conn)?;
        Ok(qc_forms::table.find(id).get_result::<ExistingQCForm>(conn)?.into())
    }).await
}

#[post("/definalize_post/<id>")]
async fn definalize_post(db: Db, id: i32, _admin: Admin) -> Result<Json<ExistingQCForm>>{
    db.run(move |conn| {
        diesel::update(qc_forms::table.find(id)).set(qc_forms::finalized.eq(false)).execute(conn)?;
        Ok(qc_forms::table.find(id).get_result::<ExistingQCForm>(conn)?.into())
    }).await
}

#[post("/new_post", data = "<post>")]
async fn new_post(db: Db, post: Json<NewQCForm>) -> Result<Created<Json<ExistingQCForm>>> {

    let post: Json<ExistingQCForm> = db
        .run(move |conn| {
            let count: i64 = qc_forms::table.filter(
                qc_forms::asm_serial.eq(&post.asm_serial)).count().get_result(conn)?;
            if count > 0{
                return Err(DataBaseError::ExistingAsmSerial)
            }
            let count: i64 = qc_forms::table.filter(
                qc_forms::item_serial.eq(&post.item_serial)).count().get_result(conn)?;
            if count > 0{
                return Err(DataBaseError::ExistingItemSerial)
            }
            
            let count: i64 = qc_forms::table.filter(
                qc_forms::oem_serial.eq(&post.oem_serial)).count().get_result(conn)?;
            if count > 0{
                return Err(DataBaseError::ExistingOemSerial)
            }

            diesel::insert_into(qc_forms::table)
                .values(&*post)
                .execute(conn)?;

            let res: ExistingQCForm = qc_forms::table
                .order(qc_forms::id.desc())
                .first(conn)?;

            Result::<Json<ExistingQCForm>>::Ok(res.into())
        })
        .await?;
    Ok(Created::new("/").body(post))
}

#[get("/timetest/<time>")]
async fn timetest(time: String) -> Result<String, String> {
    let time: Time = serde_json::from_str(&time).map_err(|f| format!("{:#?}", f))?;
    let time: String = serde_json::to_string(&time).map_err(|f| format!("{:#?}", f))?;
    Ok(time)
}

#[post("/update_post/<id>", data = "<update>")]
async fn update_post(
    db: Db,
    id: i32,
    mut update: Json<QCFormUpdate>,
) -> Result<Accepted<Json<ExistingQCForm>>> {
    update.last_updated = Some(time_default());
    let res: ExistingQCForm = db
        .run(move |conn| {
            let finalized: bool = qc_forms::table.find(id).select(qc_forms::finalized).get_result(conn)?;
            
            if finalized{
                return Err(DataBaseError::UpdatedFinalized);
            }

            diesel::update(qc_forms::table.filter(qc_forms::id.eq(id)))
                .set(&*update)
                .execute(conn)?;
            Ok(qc_forms::table.filter(qc_forms::id.eq(id)).first(conn)?)
        })
        .await?;
    Ok(Accepted(Json(res)))
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Databse", |rocket| async {
        rocket
            .attach(Db::fairing())
            .attach(AdHoc::on_ignite("Diesel Migrations", run_migrations))
            .mount(
                "/api",
                routes![get_post, new_post, update_post, search, timetest, finalize_post, definalize_post, delete_post],
            )
    })
}

async fn run_migrations(rocket: Rocket<Build>) -> Rocket<Build> {
    use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

    const MIGRATIONS: EmbeddedMigrations = embed_migrations!("db/diesel/migrations");

    Db::get_one(&rocket)
        .await
        .expect("database connection")
        .run(|conn| {
            conn.run_pending_migrations(MIGRATIONS)
                .expect("diesel migrations");
        })
        .await;

    rocket
}

#[allow(warnings)]
mod tests {
    use std::collections::HashMap;

    use diesel::RunQueryDsl;
    use rocket::{http::Status, local::blocking::Client};
    use time::OffsetDateTime;

    use crate::{
        database::{NewQCForm, Time},
        qc_checklist::{QCChecklist, QuestionAnswer, QuestionAnswers},
        schema::qc_forms::{self, drive_type},
    };

    use super::Db;

    type Result<T, E = rocket::response::Debug<diesel::result::Error>> = std::result::Result<T, E>;

    #[delete("/")]
    async fn destroy(db: Db) -> Result<()> {
        db.run(move |conn| diesel::delete(qc_forms::table).execute(conn))
            .await?;
        Ok(())
    }

    #[test]
    fn fuzz_data() {
        let conf = crate::Config::load_from_file("./config.json5")
            .expect("Failed to load config file. Fatial Error")
            .0;

        #[derive(Debug, Rand, Copy, Clone)]
        enum SerialStart {
            SHID = 0,
            UHEHD = 1,
            UHLTM2 = 2,
            UHLTHD = 3,
            UHLTNV = 4,
            SLOD = 5,
            SLOL = 6,
        }

        #[derive(Debug, Rand)]
        enum Initial {
            PT,
            CC,
            HQ,
            MA,
            LP,
            FH,
        }

        #[derive(Debug, Rand, Copy, Clone)]
        enum make_model {
            DELL_200,
            DELL_201,
            DELL_300,
            DELL_400,
            HP_1,
            HP_2,
            HP_3,
            HP_4,
            HP_9,
            HP_99,
        }

        let mut ids = [1u64; 7];
        let mut asm_serial_num = 1;

        use rand::{distributions::Standard, rngs::ThreadRng, Rng};
        use rand_derive::Rand;

        let rocket = rocket::build()
            .attach(super::stage())
            .mount("/api", routes![destroy]);
        let client = Client::tracked(rocket).unwrap();
        assert_eq!(client.delete("/api").dispatch().status(), Status::Ok);

        let mut rng = rand::thread_rng();
        let rng = &mut rng;

        for _ in 0..50000 {
            fn random_str<T: std::fmt::Debug>(rng: &mut ThreadRng) -> String
            where
                Standard: rand::prelude::Distribution<T>,
            {
                format!("{:?}", rng.gen::<T>())
            }

            fn random_val(rng: &mut ThreadRng, name: &str, vals: &serde_json::Value) -> String {
                let arr = vals
                    .get(name)
                    .unwrap()
                    .get("order")
                    .unwrap()
                    .as_array()
                    .unwrap();
                let index = rng.gen_range(0, arr.len());
                arr[index].as_str().unwrap().to_owned()
            }

            fn random_question(rng: &mut ThreadRng) -> QuestionAnswer {
                let rand = rng.gen_range(0.0, 1.0);

                match rand {
                    0.0..=0.7 => QuestionAnswer::Pass,
                    0.0..=0.8 => QuestionAnswer::Fail,
                    0.0..=0.9 => QuestionAnswer::NA,
                    0.0..=1.0 => QuestionAnswer::Incomplete,
                    _ => QuestionAnswer::Incomplete,
                }
            }

            let drive_size = random_val(rng, "drive_sizes", &conf);
            let sales_order = String::new();
            let build_type = random_val(rng, "build_types", &conf);

            let form = NewQCForm {
                creation_date: Time(
                    OffsetDateTime::from_unix_timestamp(rng.gen_range(
                        time::Date::MIN.midnight().assume_utc().unix_timestamp(),
                        time::Date::MAX.midnight().assume_utc().unix_timestamp(),
                    ))
                    .unwrap(),
                ),
                last_updated: Time(
                    OffsetDateTime::from_unix_timestamp(rng.gen_range(
                        time::Date::MIN.midnight().assume_utc().unix_timestamp(),
                        time::Date::MAX.midnight().assume_utc().unix_timestamp(),
                    ))
                    .unwrap(),
                ),
                build_location: random_val(rng, "build_locations", &conf),
                drive_type: random_val(rng, "drive_types", &conf),
                item_serial: {
                    let kind = rng.gen::<SerialStart>();
                    let range = if rng.gen_range(0.0, 1.0) < 0.1 {
                        rng.gen_range(1, 100)
                    } else {
                        1
                    };
                    ids[kind as usize] += range;
                    format!("{:?}-{:07}", kind, ids[kind as usize])
                },
                asm_serial: {
                    let range = if rng.gen_range(0.0, 1.0) < 0.1 {
                        rng.gen_range(1, 100)
                    } else {
                        1
                    };
                    asm_serial_num += range;

                    #[derive(Debug, Rand, Copy, Clone)]
                    enum Kind {
                        CFS,
                        OTR,
                    }

                    #[derive(Debug, Rand, Copy, Clone)]
                    enum Package {
                        LT_300U,
                        LT_200U,
                        LT_100U,
                        DT_100U,
                        DT_200U,
                        DT_300U,
                        DT_400U,
                    }

                    Some(format!(
                        "{:?}-{:?}-{:06}",
                        rng.gen::<Kind>(),
                        rng.gen::<Package>(),
                        asm_serial_num
                    ))
                },
                oem_serial: {
                    rand::thread_rng()
                        .sample_iter(&rand::distributions::Alphanumeric)
                        .take(7)
                        .map(char::from)
                        .collect()
                },
                make_model: random_str::<make_model>(rng),
                mso_installed: rng.gen::<bool>(),
                operating_system: random_val(rng, "operating_systems", &conf),
                processor_gen: random_val(rng, "processor_gens", &conf),
                processor_type: random_val(rng, "processor_types", &conf),
                qc_answers: {
                    let mut checks = QCChecklist::new();

                    for (id, check) in conf["qc_checks"]["questions"].as_object().unwrap().iter() {
                        if check
                            .get("whitelist_build_types")
                            .map(|f| {
                                f.as_array()
                                    .map(|f| {
                                        f.contains(&serde_json::Value::String(build_type.clone()))
                                    })
                                    .unwrap()
                            })
                            .unwrap_or(true)
                        {
                            checks.0.insert(
                                id.to_owned(),
                                QuestionAnswers([random_question(rng), random_question(rng)]),
                            );
                        }
                    }
                    checks
                },
                qc1_initial: random_str::<Initial>(rng),
                qc2_initial: if rng.gen::<bool>() {
                    None
                } else {
                    Some(random_str::<Initial>(rng))
                },
                ram_size: random_val(rng, "ram_sizes", &conf),
                ram_type: random_val(rng, "ram_types", &conf),
                drive_size,
                sales_order: None,
                tech_notes: "".into(),
                metadata: None,
                build_type,
                finalized: false,
            };

            assert_eq!(
                client.post("/api/new_post").json(&form).dispatch().status(),
                Status::Created
            );
        }
    }
}
