use std::str::Chars;

use diesel::sql_types::TimestamptzSqlite;
use diesel::sqlite::Sqlite;
use rocket::fairing::AdHoc;
use rocket::response::{status::Created, Debug};
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::{Build, Rocket};

use rocket_sync_db_pools::diesel;
use serde_json::Value;
use time::PrimitiveDateTime;

use crate::qurry_builder::ExpressionParser;

use self::diesel::prelude::*;

#[database("diesel")]
pub struct Db(diesel::SqliteConnection);

impl Db {
    pub async fn validate_user(&self, email: String, password: String) -> Option<i32> {
        self.run(|db| {
            user_accounts::table
                .filter(
                    user_accounts::email
                        .eq(email)
                        .and(user_accounts::password.eq(password)),
                )
                .select(user_accounts::id)
                .first(db)
                .ok()
                .flatten()
        })
        .await
    }
}

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

type DynTable = diesel_dynamic_schema::Table<String>;
type DynExpr =
    Box<dyn BoxableExpression<qc_forms::table, Sqlite, SqlType = diesel::sql_types::Bool>>;

#[get("/test/<search>")]
async fn list_search(db: Db, search: Option<&str>) -> Result<Json<Vec<QCForm>>> {

    struct VisitorTest{

    }
    impl VisitorTest{
        pub fn new() -> Self{
            Self{}
        }
    }
    impl crate::qurry_builder::Visitor<DynExpr> for VisitorTest{
        fn eq(&mut self, ident: String, value: String) -> DynExpr{
            // format!("({}={:#?})", ident, value)
            dyn_qc_form_column!(ident.as_str(), column, {
                Box::new(column.eq(value))
            }, {todo!()}, {todo!()}, {
                todo!()   
            })
        }
        fn lt(&mut self, ident: String, value: String) -> DynExpr{
            dyn_qc_form_column!(ident.as_str(), column, {
                Box::new(column.lt(value))
            }, {todo!()}, {todo!()}, {
                todo!()   
            })
        }
        fn gt(&mut self, ident: String, value: String) -> DynExpr{
            dyn_qc_form_column!(ident.as_str(), column, {
                Box::new(column.gt(value))
            }, {todo!()}, {todo!()}, {
                todo!()   
            })
        }
        fn colon(&mut self, ident: String, value: String) -> DynExpr{
            dyn_qc_form_column!(ident.as_str(), column, {
                Box::new(column.like(value))
            }, {todo!()}, {todo!()}, {
                todo!()   
            })
        }

        fn or(&mut self, ls: DynExpr, rs: DynExpr) -> DynExpr{
            Box::new(ls.or(rs))
        }

        fn and(&mut self, ls: DynExpr, rs: DynExpr) -> DynExpr{
            Box::new(ls.and(rs))
        }

    }


    let mut boxed = qc_forms::table
        .order_by(qc_forms::id.asc())
        .limit(100)
        .into_boxed();

    let mut visitor = VisitorTest::new();
    if let Some(search) = search{
        let res = ExpressionParser::new(search, &mut visitor).parse();
    
        match res{
            Ok(ok) => {
                boxed = boxed.filter(ok);
            }
            Err(err) => {
                todo!("{:#?}", err)
            }
        }
    }
    // drop(expr);
    // println!("{:#?}", res);


    // let res: Box<
    //     dyn BoxableExpression<
    //         qc_forms::table,
    //         Sqlite,
    //         SqlType = diesel::expression::expression_types::NotSelectable,
    //     >,
    // > = dyn_qc_form_column!("test", column, { Box::new(column.asc()) }, { todo!() });


    // use diesel_dynamic_schema::table;

    // let bruh: diesel::sql_types::TimestamptzSqlite;

    // let tabel = table("qc_forms");
    // let comumn = tabel.column::<diesel::sql_types::Text, _>("processortype");

    // // qc_forms::processortype.
    // // let mut boxed_thing = Box::new(qc_forms::processorgen);
    // // boxed_thing = Box::new(qc_forms::processortype);

    // // boxed = boxed.filter(boxed_thing.like("other"));
    // let res: Box<
    //     dyn BoxableExpression<qc_forms::table, Sqlite, SqlType = diesel::sql_types::Bool>,
    // > = Box::new(qc_forms::processortype.like("other"));

    // boxed = boxed.filter(res);
    // boxed = boxed.filter(qc_forms::processortype.like("other"));
    // boxed = boxed.filter(qc_forms::salesorder.like("other"));
    // boxed = boxed.filter(qc_forms::salesorder.like("other"));

    // let fucked: DynExpr = Box::new(qc_forms::processortype.like("other"));
    // let fucked_2 = Box::new(qc_forms::processortype.like("other"));
    // let totally_fucked: DynExpr = Box::new(fucked.or(fucked_2));

    // boxed = boxed.filter(totally_fucked);

    // let kind = 1;
    // let name = "";
    // let table: diesel_dynamic_schema::Column<diesel_dynamic_schema::Table<qc_forms::table>, &str, diesel::sql_types::Text> = table(qc_forms::table).column::<diesel::sql_types::Text, _>("as");
    // let stupid = match name{
    //     "bruh" => {
    //         qc_forms::assemblydate
    //     },
    //     _ => {
    //         qc_forms::buildtype
    //     }
    // };
    // // None?
    // match kind {
    //     0 => {
    //         tabel
    //     }

    //     _ => {

    //     }
    // }carog

    let qc_posts: Vec<QCForm> = db.run(move |conn| boxed.load(conn)).await?;

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
