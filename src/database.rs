use diesel::sqlite::Sqlite;
use rocket::fairing::AdHoc;
use rocket::form::Form;
use rocket::response::{status::Created, Debug};
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::{Build, Rocket};

use rocket_sync_db_pools::diesel;
use serde_json::Value;

use crate::qurry_builder::{ExpressionParser};

use self::diesel::prelude::*;

#[database("diesel")]
pub struct Db(diesel::SqliteConnection);


type Result<T, E = Debug<diesel::result::Error>> = std::result::Result<T, E>;

#[derive(Debug, Clone, Deserialize, Serialize, Queryable, Insertable)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = posts)]
struct Post {
    #[serde(skip_deserializing)]
    id: Option<i32>,
    title: String,
    text: String,
    #[serde(skip_deserializing)]
    published: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, Queryable, Insertable)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = qc_forms)]
struct QCForm {
    #[serde(skip_deserializing)]
    id: Option<i32>,
    #[serde(with = "time::serde::iso8601")]
    #[serde(default = "time_default")]
    assemblydate: time::OffsetDateTime,
    buildlocation: String,
    buildtype: String,
    drivetype: String,
    itemserial: String,
    makemodel: String,
    msoinstalled: String,
    operatingsystem: String,
    processorgen: String,
    processortype: String,
    qc1: String,
    qc1initial: String,
    qc2: String,
    qc2initial: String,

    ramsize: String,
    ramtype: String,
    rctpackage: String,
    salesorder: String,
    technotes: String,
}

fn time_default() -> time::OffsetDateTime {
    time::OffsetDateTime::now_utc()
}

#[derive(Debug, Clone, Deserialize, Serialize, Queryable, Insertable)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = user_accounts)]
struct UserAccount {
    #[serde(skip_deserializing)]
    id: Option<i32>,
    email: String,
    password: String,
}

macro_rules! dyn_qc_form_column {
    ($column:expr, $ident:ident, $succ:block, $fail:block) => {
        dyn_qc_form_column!($column, $ident, $succ, $succ, $succ, $fail)
    };
    ($column:expr, $ident:ident, $succ:block, $succ_id:block, $succ_date:block, $fail:block) => {
        match $column {
            "id" => {
                let $ident = qc_forms::id;
                $succ_id
            }
            "assemblydate" => {
                let $ident = qc_forms::assemblydate;
                $succ_date
            }
            "buildlocation" => {
                let $ident = qc_forms::buildlocation;
                $succ
            }
            "buildtype" => {
                let $ident = qc_forms::buildtype;
                $succ
            }
            "drivetype" => {
                let $ident = qc_forms::drivetype;
                $succ
            }
            "itemserial" => {
                let $ident = qc_forms::itemserial;
                $succ
            }
            "makemodel" => {
                let $ident = qc_forms::makemodel;
                $succ
            }
            "msoinstalled" => {
                let $ident = qc_forms::msoinstalled;
                $succ
            }
            "operatingsystem" => {
                let $ident = qc_forms::operatingsystem;
                $succ
            }
            "processorgen" => {
                let $ident = qc_forms::processorgen;
                $succ
            }
            "processortype" => {
                let $ident = qc_forms::processortype;
                $succ
            }
            "qc1" => {
                let $ident = qc_forms::qc1;
                $succ
            }
            "qc1initial" => {
                let $ident = qc_forms::qc1initial;
                $succ
            }
            "qc2" => {
                let $ident = qc_forms::qc2;
                $succ
            }
            "qc2initial" => {
                let $ident = qc_forms::qc2initial;
                $succ
            }

            "ramsize" => {
                let $ident = qc_forms::ramsize;
                $succ
            }
            "ramtype" => {
                let $ident = qc_forms::ramtype;
                $succ
            }
            "rctpackage" => {
                let $ident = qc_forms::rctpackage;
                $succ
            }
            "salesorder" => {
                let $ident = qc_forms::salesorder;
                $succ
            }
            "technotes" => {
                let $ident = qc_forms::technotes;
                $succ
            }
            _ => $fail,
        }
    };
}

table! {
    qc_forms(id) {
        id -> Nullable<Integer>,
        // assembly_date -> Text,
        assemblydate -> TimestamptzSqlite,
        buildlocation -> Text,
        buildtype -> Text,
        drivetype -> Text,
        itemserial -> Text,
        makemodel -> Text,
        msoinstalled -> Text,
        operatingsystem -> Text,
        processorgen -> Text,
        processortype -> Text,
        qc1 -> Text,
        qc1initial -> Text,
        qc2 -> Text,
        qc2initial -> Text,

        ramsize -> Text,
        ramtype -> Text,
        rctpackage -> Text,
        salesorder -> Text,
        technotes -> Text,
    }
}

table! {
    posts (id) {
        id -> Nullable<Integer>,
        title -> Text,
        text -> Text,
        published -> Bool,
    }
}
table! {
    user_accounts(id){
        id -> Nullable<Integer>,
        email -> Text,
        password -> Text,
    }
}

// #[post("/", data = "<post>")]
// async fn create(db: Db, post: Json<Post>) -> Result<Created<Json<Post>>> {
//     let post_value = post.clone();
//     db.run(move |conn| {
//         diesel::insert_into(posts::table)
//             .values(&*post_value)
//             .execute(conn)
//     })
//     .await?;

//     Ok(Created::new("/").body(post))
// }

#[post("/", data = "<post>")]
async fn create(db: Db, post: Json<QCForm>) -> Result<Created<Json<QCForm>>> {
    let post_value = post.clone();
    db.run(move |conn| {
        diesel::insert_into(qc_forms::table)
            .values(&*post_value)
            .execute(conn)
    })
    .await?;
    Ok(Created::new("/").body(post))
}

// #[get("/")]
// async fn list(db: Db) -> Result<Json<Vec<Option<i32>>>> {
//     let ids: Vec<Option<i32>> = db
//         .run(move |conn| qc_forms::table.select(qc_forms::id).load(conn))
//         .await?;

//     Ok(Json(ids))
// }

#[get("/")]
async fn list(db: Db) -> Result<Json<Vec<QCForm>>> {
    let qc_posts: Vec<QCForm> = db.run(move |conn| qc_forms::table.load(conn)).await?;

    Ok(qc_posts.into())
}

// type DynTable = diesel_dynamic_schema::Table<String>;
type DynExpr =
    Box<dyn BoxableExpression<qc_forms::table, Sqlite, SqlType = diesel::sql_types::Bool>>;

#[derive(Responder)]
#[response(status = 500, content_type = "json")]
enum QuerryError {
    DieselError(Debug<diesel::result::Error>),
    OtherError(Value),
}

type VisitorError = Value;

struct VisitorTest {}
impl VisitorTest {
    pub fn new() -> Self {
        Self {}
    }
}
impl crate::qurry_builder::Visitor<DynExpr, VisitorError> for VisitorTest {
    fn eq(&mut self, ident: String, value: String) -> Result<DynExpr, VisitorError> {
        dyn_qc_form_column!(
            ident.as_str(),
            column,
            { Ok(Box::new(column.eq(value))) },
            { todo!() },
            { todo!() },
            {
                Err(serde_json::json!({
                    "Error": format!("Invalid tabel selected for ordering: {}", ident)
                }))
            }
        )
    }
    fn lt(&mut self, ident: String, value: String) -> Result<DynExpr, VisitorError> {
        dyn_qc_form_column!(
            ident.as_str(),
            column,
            { Ok(Box::new(column.lt(value))) },
            { todo!() },
            { todo!() },
            {
                Err(serde_json::json!({
                    "Error": format!("Invalid tabel selected for ordering: {}", ident)
                }))
            }
        )
    }
    fn gt(&mut self, ident: String, value: String) -> Result<DynExpr, VisitorError> {
        dyn_qc_form_column!(
            ident.as_str(),
            column,
            { Ok(Box::new(column.gt(value))) },
            { todo!() },
            { todo!() },
            {
                Err(serde_json::json!({
                    "Error": format!("Invalid tabel selected for ordering: {}", ident)
                }))
            }
        )
    }
    fn colon(&mut self, ident: String, value: String) -> Result<DynExpr, VisitorError> {
        dyn_qc_form_column!(
            ident.as_str(),
            column,
            { Ok(Box::new(column.like(value))) },
            { todo!() },
            { todo!() },
            {
                Err(serde_json::json!({
                    "Error": format!("Invalid tabel selected for ordering: {}", ident)
                }))
            }
        )
    }

    fn or(&mut self, ls: DynExpr, rs: DynExpr) -> Result<DynExpr, VisitorError> {
        Ok(Box::new(ls.or(rs)))
    }

    fn and(&mut self, ls: DynExpr, rs: DynExpr) -> Result<DynExpr, VisitorError> {
        Ok(Box::new(ls.and(rs)))
    }
}

#[derive(FromForm)]
struct SearchForm<'f> {
    limit: Option<i64>,
    search: Option<&'f str>,
    order_table: Option<&'f str>,
    ascending: Option<bool>,
}

#[post("/search", data = "<search>")]
async fn list_search(
    db: Db,
    search: Form<SearchForm<'_>>,
) -> std::result::Result<Json<Vec<QCForm>>, QuerryError> {
    let mut boxed = if let Some(limit) = search.limit {
        qc_forms::table.limit(limit).into_boxed()
    } else {
        qc_forms::table.into_boxed()
    };

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
                    return Err(QuerryError::OtherError(
                        serde_json::to_value(err).unwrap_or_default(),
                    ))
                }
            }
        }
    }
    if let Some(order_table) = search.order_table {
        dyn_qc_form_column!(
            order_table,
            column,
            {
                if search.ascending.unwrap_or(true) {
                    boxed = boxed.order_by(column.asc())
                } else {
                    boxed = boxed.order_by(column.desc())
                }
            },
            {
                return Err(QuerryError::OtherError(serde_json::json!({
                    "Error": format!("Invalid tabel selected for ordering: {}", order_table)
                })));
            }
        );
    } else {
        boxed = if search.ascending.unwrap_or(true) {
            boxed.order_by(qc_forms::id.asc())
        } else {
            boxed.order_by(qc_forms::id.desc())
        }
    }

    let qc_posts: Vec<QCForm> = match db.run(move |conn| boxed.load(conn)).await {
        Ok(ok) => ok,
        Err(err) => return Err(QuerryError::DieselError(err.into())),
    };

    Ok(qc_posts.into())
}

#[get("/<id>")]
async fn read(db: Db, id: i32) -> Option<Json<QCForm>> {
    db.run(move |conn| qc_forms::table.filter(qc_forms::id.eq(id)).first(conn))
        .await
        .map(Json)
        .ok()
}

#[delete("/<id>")]
async fn delete(db: Db, id: i32) -> Result<Option<()>> {
    let affected = db
        .run(move |conn| {
            diesel::delete(posts::table)
                .filter(posts::id.eq(id))
                .execute(conn)
        })
        .await?;

    Ok((affected == 1).then_some(()))
}

#[delete("/")]
async fn destroy(db: Db) -> Result<()> {
    db.run(move |conn| diesel::delete(posts::table).execute(conn))
        .await?;

    Ok(())
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

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Diesel SQLite Stage", |rocket| async {
        rocket
            .attach(Db::fairing())
            .attach(AdHoc::on_ignite("Diesel Migrations", run_migrations))
            .mount(
                "/api",
                routes![list, read, create, delete, destroy, list_search],
            )
    })
}
