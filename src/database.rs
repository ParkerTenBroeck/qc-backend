use diesel::sqlite::Sqlite;
use rocket::fairing::AdHoc;
use rocket::form::Form;
use rocket::response::status::Accepted;
use rocket::response::{status::Created, Debug};
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::{Build, Rocket};

use rocket_sync_db_pools::diesel;
use serde_json::Value;

use crate::qc_checklist::QCChecklist;
use crate::qurry_builder::ExpressionParser;
use crate::time::Time;

use self::diesel::prelude::*;

use crate::schema::*;

#[database("diesel")]
pub struct Db(diesel::SqliteConnection);

type Result<T, E = Debug<diesel::result::Error>> = std::result::Result<T, E>;

fn time_default() -> Time {
    Time(time::OffsetDateTime::now_utc())
}

#[derive(Debug, Clone, Deserialize, Serialize, Queryable, Insertable, AsChangeset)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = crate::schema::qc_forms)]
struct QCForm {
    #[serde(skip_deserializing)]
    id: Option<i32>,
    #[serde(default = "time_default")]
    assemblydate: Time,
    buildlocation: String,
    buildtype: String,
    drivetype: String,
    itemserial: String,
    makemodel: String,
    msoinstalled: bool,
    operatingsystem: String,
    processorgen: String,
    processortype: String,
    qc1: QCChecklist,
    qc1initial: String,
    qc2: QCChecklist,
    qc2initial: String,

    ramsize: String,
    ramtype: String,
    rctpackage: String,
    salesorder: String,
    technotes: String,
}

macro_rules! dyn_qc_form_column {
    ($column:expr, $ident:ident, $succ:block, $fail:block) => {
        dyn_qc_form_column!($column, $ident, $succ, $succ, $succ, $succ, $fail)
    };
    ($column:expr, $ident:ident, $succ_text:block, $succ_id:block, $succ_date:block, $succ_bool:block, $fail:block) => {
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
                $succ_text
            }
            "ramsize" => {
                let $ident = qc_forms::ramsize;
                $succ_text
            }
            "ramtype" => {
                let $ident = qc_forms::ramtype;
                $succ_text
            }
            "rctpackage" => {
                let $ident = qc_forms::rctpackage;
                $succ_text
            }
            "salesorder" => {
                let $ident = qc_forms::salesorder;
                $succ_text
            }
            "technotes" => {
                let $ident = qc_forms::technotes;
                $succ_text
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
        dyn_qc_form_column!(
            ident.as_str(),
            column,
            { Ok(Box::new(column.eq(value))) },
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

        if !order_table.trim().is_empty() {
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
async fn new_post(db: Db, post: Json<QCForm>) -> Result<Created<Json<QCForm>>> {
    let post_value = post.clone();
    db.run(move |conn| {
        diesel::insert_into(qc_forms::table)
            .values(&*post_value)
            .execute(conn)
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

#[post("/overwrite_post", data = "<post>")]
async fn overwrite_post(db: Db, post: Json<QCForm>) -> Result<Accepted<Json<QCForm>>> {
    let post_value = post.clone();
    db.run(move |conn| {
        diesel::update(qc_forms::table.filter(qc_forms::id.eq(post_value.id)))
            .set(&*post_value)
            .execute(conn)
    })
    .await?;
    Ok(Accepted(Some(post)))
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
        qc_checklist::QCChecklist,
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
        #[derive(Debug, Rand)]
        enum BuildType {
            Laptop,
            Desktop_Mini,
            Desktop_Micro,
            Desktop_Standard,
        }

        #[derive(Debug, Rand)]
        enum Location {
            NIA,
            MIA,
        }

        #[derive(Debug, Rand)]
        enum DriveType {
            SSD,
            M2,
            NVMe,
            HDD,
        }

        #[derive(Debug, Rand)]
        enum OsInstalled {
            Linux,
            Windows10,
            Windows11,
            ChromOs,
            Android,
        }

        #[derive(Debug, Rand)]
        enum ProcessorType {
            Corei5,
            Corei3,
            Corei7,
            Corei9,
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

        #[derive(Debug, Rand)]
        enum RamType {
            DDR2,
            DDR3,
            DDR4,
            DDR5,
        }

        #[derive(Debug, Rand)]
        enum RamSize {
            GB1,
            GB2,
            GB4,
            GB8,
            GB16,
            GB32,
            GB64,
        }

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

        #[derive(Debug, Rand, Copy, Clone)]
        enum RCTPackage {
            LT_300U,
            LT_200U,
            LT_100U,
            DT_100U,
            DT_200U,
            DT_300U,
            DT_400U,
        }

        #[derive(Debug, Rand, Copy, Clone)]
        enum SalesOrder {
            CFS,
            OTR,
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

        let check_ids = [
            "bios_pass",
            "usb_posts",
            "bios_reset",
            "keybaord_mouse",
            "case",
            "cd_dvd_drive",
            "device_manager",
            "image_loaded",
        ];

        let mut ids = [1u64; 7];

        use rand::{distributions::Standard, rngs::ThreadRng, Rng};
        use rand_derive::Rand;

        let rocket = rocket::build()
            .attach(super::stage())
            .mount("/api", routes![destroy]);
        let client = Client::tracked(rocket).unwrap();
        assert_eq!(client.delete("/api").dispatch().status(), Status::Ok);

        let mut rng = rand::thread_rng();
        let rng = &mut rng;
        for _ in 0..500000 {
            fn random_str<T: std::fmt::Debug>(rng: &mut ThreadRng) -> String
            where
                Standard: rand::prelude::Distribution<T>,
            {
                format!("{:?}", rng.gen::<T>())
            }

            let form = QCForm {
                id: None,
                assemblydate: Time(
                    OffsetDateTime::from_unix_timestamp(rng.gen_range(
                        time::Date::MIN.midnight().assume_utc().unix_timestamp(),
                        time::Date::MAX.midnight().assume_utc().unix_timestamp(),
                    ))
                    .unwrap(),
                ),
                buildlocation: random_str::<Location>(rng),
                buildtype: random_str::<BuildType>(rng),
                drivetype: random_str::<DriveType>(rng),
                itemserial: {
                    let kind = rng.gen::<SerialStart>();
                    let range = if rng.gen_range(0.0, 1.0) < 0.1 {
                        rng.gen_range(1, 100)
                    } else {
                        1
                    };
                    ids[kind as usize] += range;
                    format!("{:?}-{:80}", kind, ids[kind as usize])
                },
                makemodel: random_str::<MakeModel>(rng),
                msoinstalled: rng.gen::<bool>(),
                operatingsystem: random_str::<OsInstalled>(rng),
                processorgen: format!("{}", rng.gen_range(1, 14)),
                processortype: random_str::<ProcessorType>(rng),
                qc1: {
                    let mut checks = QCChecklist::new();
                    for check in check_ids {
                        checks.0.insert(check.to_owned(), rng.gen_range(0, 4));
                    }
                    checks
                },
                qc1initial: random_str::<Initial>(rng),
                qc2: {
                    let mut checks = QCChecklist::new();
                    for check in check_ids {
                        checks.0.insert(check.to_owned(), rng.gen_range(0, 4));
                    }
                    checks
                },
                qc2initial: random_str::<Initial>(rng),
                ramsize: random_str::<RamSize>(rng),
                ramtype: random_str::<RamType>(rng),
                rctpackage: random_str::<RCTPackage>(rng),
                salesorder: random_str::<SalesOrder>(rng),
                technotes: "".into(),
            };

            assert_eq!(
                client.post("/api/new_post").json(&form).dispatch().status(),
                Status::Created
            );
        }
    }
}
