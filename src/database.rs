use diesel::sqlite::Sqlite;
use rocket::fairing::AdHoc;
use rocket::form::Form;
use rocket::response::status::Accepted;
use rocket::response::{status::Created, Debug};
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::{Build, Rocket};
use crate::json_text::JsonText;

use rocket_sync_db_pools::diesel;
use serde_json::Value;

use crate::qc_checklist::QCChecklist;
use crate::qurry_builder::ExpressionParser;
use crate::time::Time;

use self::diesel::prelude::*;

use crate::schema::*;

#[database("diesel")]
pub struct Db(diesel::SqliteConnection);

impl Db {
    pub async fn get_form(&self, id: i32) -> Result<QCForm> {
        let form: QCForm = self
            .run(move |conn| qc_forms::table.filter(qc_forms::id.eq(id)).first(conn))
            .await?;
        Ok(form)
    }
}

pub type Result<T, E = Debug<diesel::result::Error>> = std::result::Result<T, E>;

fn time_default() -> Time {
    Time(time::OffsetDateTime::now_utc())
}

#[derive(Debug, Clone, Deserialize, Serialize, Queryable, Insertable, AsChangeset)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = crate::schema::qc_forms)]
pub struct QCForm {
    #[serde(skip_deserializing)]
    pub id: Option<i32>,
    #[serde(default = "time_default")]
    pub assemblydate: Time,
    pub buildlocation: String,
    pub buildtype: String,
    pub drivetype: String,
    // example SHIL-0023746
    pub itemserial: String,
    // example CFS-SL300F-001220
    pub asmserial: Option<String>,
    // the serial listed in the bios of the device
    pub oemserial: String,
    pub makemodel: String,
    pub msoinstalled: bool,
    pub operatingsystem: String,
    pub processorgen: String,
    pub processortype: String,
    pub qc1: QCChecklist,
    pub qc1initial: String,
    pub qc2: QCChecklist,
    pub qc2initial: Option<String>,

    pub ramsize: String,
    pub ramtype: String,

    pub salesorder: Option<String>,
    pub drivesize: String,
    pub technotes: String,

    pub metadata: Option<JsonText>,
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
            "assemblydate" => {
                let $ident = qc_forms::assemblydate;
                $succ_date
            }
            "buildlocation" => {
                let $ident = qc_forms::buildlocation;
                $succ_text
            }
            "buildtype" => {
                let $ident = qc_forms::buildtype;
                $succ_text
            }
            "drivetype" => {
                let $ident = qc_forms::drivetype;
                $succ_text
            }
            "itemserial" => {
                let $ident = qc_forms::itemserial;
                $succ_text
            }
            "asmserial" => {
                let $ident = qc_forms::asmserial;
                $succ_text_optional
            }
            "oemserial" => {
                let $ident = qc_forms::oemserial;
                $succ_text
            }
            "makemodel" => {
                let $ident = qc_forms::makemodel;
                $succ_text
            }
            "msoinstalled" => {
                let $ident = qc_forms::msoinstalled;
                $succ_bool
            }
            "operatingsystem" => {
                let $ident = qc_forms::operatingsystem;
                $succ_text
            }
            "processorgen" => {
                let $ident = qc_forms::processorgen;
                $succ_text
            }
            "processortype" => {
                let $ident = qc_forms::processortype;
                $succ_text
            }
            "qc1" => {
                let $ident = qc_forms::qc1;
                $succ_text
            }
            "qc1initial" => {
                let $ident = qc_forms::qc1initial;
                $succ_text
            }
            "qc2" => {
                let $ident = qc_forms::qc2;
                $succ_text
            }
            "qc2initial" => {
                let $ident = qc_forms::qc2initial;
                $succ_text_optional
            }
            "ramsize" => {
                let $ident = qc_forms::ramsize;
                $succ_text
            }
            "ramtype" => {
                let $ident = qc_forms::ramtype;
                $succ_text
            }
            "drivesize" => {
                let $ident = qc_forms::drivesize;
                $succ_text
            }
            "salesorder" => {
                let $ident = qc_forms::salesorder;
                $succ_text_optional
            }
            "technotes" => {
                let $ident = qc_forms::technotes;
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

macro_rules! unwrap_visitor_exression {
    ($type:ty, $expr:expr) => {{
        let val: $type = match serde_json::from_str(&$expr) {
            Ok(ok) => ok,
            Err(err) => return Err(serde_json::json!({ "Error": format!("{:?}", err) })),
        };
        val
    }};
}

impl crate::qurry_builder::Visitor<DynExpr, VisitorError> for VisitorTest {
    fn eq(&mut self, ident: String, value: String) -> Result<DynExpr, VisitorError> {
        // {
        //     let test = diesel_dynamic_schema::table("").column::<diesel::sql_types::Text, _>("test");
        //     test.eq(value);
        // }
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
            { Ok(Box::new(column.eq(unwrap_visitor_exression!(Time, value)))) },
            { Ok(Box::new(column.eq(unwrap_visitor_exression!(bool, value)),)) },
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
            {
                if value.is_empty() {
                    Err(serde_json::json!({
                        "Error": "given value cannot be null for lt expression"
                    }))
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
            { Ok(Box::new(column.lt(unwrap_visitor_exression!(Time, value)))) },
            { Ok(Box::new(column.lt(unwrap_visitor_exression!(bool, value)),)) },
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
            {
                if value.is_empty() {
                    Err(serde_json::json!({
                        "Error": "given value cannot be null for gt expression"
                    }))
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
            { Ok(Box::new(column.gt(unwrap_visitor_exression!(Time, value)))) },
            { Ok(Box::new(column.gt(unwrap_visitor_exression!(bool, value)),)) },
            {
                Err(serde_json::json!({
                    "Error": format!("Invalid tabel selected for ordering: {}", ident)
                }))
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
                    unwrap_visitor_exression!(Time, low_value),
                    unwrap_visitor_exression!(Time, high_value),
                )))
            },
            {
                Ok(Box::new(column.between(
                    unwrap_visitor_exression!(bool, low_value),
                    unwrap_visitor_exression!(bool, high_value),
                )))
            },
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
            _column,
            { Ok(Box::new(_column.like(value))) },
            {
                if value.is_empty() {
                    Err(serde_json::json!({
                        "Error": "given value cannot be null for like expression"
                    }))
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
                Err(serde_json::json!({
                    "Error": "Cannot use like operator with Option<i32> fields"
                }))
            },
            {
                Err(serde_json::json!({
                    "Error": "Cannot use like operator with DateTime fields"
                }))
            },
            {
                Err(serde_json::json!({
                    "Error": "Cannot use like operator with bool fields"
                }))
            },
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
) -> std::result::Result<Json<Vec<QCForm>>, QuerryError> {
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
                    return Err(QuerryError::OtherError(
                        serde_json::to_value(err).unwrap_or_default(),
                    ))
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
                return Err(QuerryError::OtherError(serde_json::json!({
                    "Error": format!("Invalid tabel selected for ordering: {}", order_table)
                })));
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

    let qc_posts: Vec<QCForm> = match db.run(move |conn| boxed.load(conn)).await {
        Ok(ok) => ok,
        Err(err) => return Err(QuerryError::DieselError(err.into())),
    };

    Ok(qc_posts.into())
}

#[get("/get_post/<id>")]
async fn get_post(db: Db, id: i32) -> Option<Json<QCForm>> {
    db.run(move |conn| qc_forms::table.filter(qc_forms::id.eq(id)).first(conn))
        .await
        .map(Json)
        .ok()
}

#[post("/new_post", data = "<post>")]
async fn new_post(db: Db, mut post: Json<QCForm>) -> Result<Created<Json<QCForm>>> {
    let post_value = post.clone();

    post.id = db
        .run(move |conn| {
            diesel::insert_into(qc_forms::table)
                .values(&*post_value)
                .execute(conn)?;

            let res = qc_forms::table
                .select(qc_forms::id)
                .order(qc_forms::id.desc())
                .first(conn)?;
            println!("{:#?}", res);

            Result::<Option<i32>, Debug<diesel::result::Error>>::Ok(res)
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

#[post("/overwrite_post/<id>", data = "<post>")]
async fn overwrite_post(db: Db, id: i32, mut post: Json<QCForm>) -> Result<Accepted<Json<QCForm>>> {
    post.id = Some(id);
    let post_value = post.clone();
    db.run(move |conn| {
        diesel::update(qc_forms::table.filter(qc_forms::id.eq(id)))
            .set(&*post_value)
            .execute(conn)
    })
    .await?;
    Ok(Accepted(post))
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Diesel SQLite Stage", |rocket| async {
        rocket
            .attach(Db::fairing())
            .attach(AdHoc::on_ignite("Diesel Migrations", run_migrations))
            .mount(
                "/api",
                routes![get_post, new_post, overwrite_post, search, timetest],
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
        database::{QCForm, Time},
        qc_checklist::{QCChecklist, QuestionAnswer},
        schema::qc_forms::{self, drivetype},
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
        let conf = crate::Config::load_from_file("./res/everything.json")
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
        enum MakeModel {
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

        for _ in 0..10 {
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

            let drivesize = random_val(rng, "drivesizes", &conf);
            let salesorder = String::new();
            let buildtype = random_val(rng, "buildtypes", &conf);
                

            let form = QCForm {
                id: None,
                assemblydate: Time(
                    OffsetDateTime::from_unix_timestamp(rng.gen_range(
                        time::Date::MIN.midnight().assume_utc().unix_timestamp(),
                        time::Date::MAX.midnight().assume_utc().unix_timestamp(),
                    ))
                    .unwrap(),
                ),
                buildlocation: random_val(rng, "buildlocations", &conf),
                drivetype: random_val(rng, "drivetypes", &conf),
                itemserial: {
                    let kind = rng.gen::<SerialStart>();
                    let range = if rng.gen_range(0.0, 1.0) < 0.1 {
                        rng.gen_range(1, 100)
                    } else {
                        1
                    };
                    ids[kind as usize] += range;
                    format!("{:?}-{:07}", kind, ids[kind as usize])
                },
                asmserial: {
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
                oemserial: {
                    rand::thread_rng()
                        .sample_iter(&rand::distributions::Alphanumeric)
                        .take(7)
                        .map(char::from)
                        .collect()
                },
                makemodel: random_str::<MakeModel>(rng),
                msoinstalled: rng.gen::<bool>(),
                operatingsystem: random_val(rng, "operatingsystems", &conf),
                processorgen: random_val(rng, "processorgens", &conf),
                processortype: random_val(rng, "processortypes", &conf),
                qc1: {
                    let mut checks = QCChecklist::new();
                    
                    for (id, check) in conf["qc_checks"]["questions"].as_object().unwrap().iter() {
                        if check.get("whitelist_buildtypes").map(|f|f.as_array().map(|f|f.contains(&serde_json::Value::String(buildtype.clone()))).unwrap()).unwrap_or(true){
                            checks.0.insert(id.to_owned(), QuestionAnswer::Pass);
                        }
                    }
                    checks
                },
                qc1initial: random_str::<Initial>(rng),
                qc2: {
                    let mut checks = QCChecklist::new();
                    for (id, check) in conf["qc_checks"]["questions"].as_object().unwrap().iter() {
                        if check.get("whitelist_buildtypes").map(|f|f.as_array().map(|f|f.contains(&serde_json::Value::String(buildtype.clone()))).unwrap()).unwrap_or(true){
                            checks.0.insert(id.to_owned(), random_question(rng));
                        }
                    }
                    checks
                },
                qc2initial: if rng.gen::<bool>() {
                    None
                } else {
                    Some(random_str::<Initial>(rng))
                },
                ramsize: random_val(rng, "ramsizes", &conf),
                ramtype: random_val(rng, "ramtypes", &conf),
                drivesize,
                salesorder: None,
                technotes: "".into(),
                metadata: None,
                buildtype,
            };

            assert_eq!(
                client.post("/api/new_post").json(&form).dispatch().status(),
                Status::Created
            );
        }
    }
}
