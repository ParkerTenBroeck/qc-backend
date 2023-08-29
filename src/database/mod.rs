use crate::json_text::JsonText;

use rocket::fairing::AdHoc;

use rocket::serde::{Deserialize, Serialize};
use rocket::{Build, Rocket};

use rocket_sync_db_pools::diesel;

use crate::qc_checklist::QCChecklist;

use crate::time::Time;

use self::diesel::prelude::*;

pub use self::errors::*;
use self::schema::*;

pub mod admin;
pub mod create;
pub mod errors;
pub mod schema;
pub mod search;
pub mod update;

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

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Databse", |rocket| async {
        rocket
            .attach(Db::fairing())
            .attach(AdHoc::on_ignite("Diesel Migrations", run_migrations))
            .mount(
                "/api",
                routes![
                    search::get_post,
                    create::new_post,
                    update::update_post,
                    search::search,
                    search::tokenize,
                    search::compile,
                    update::finalize_post,
                    admin::definalize_post,
                    admin::delete_post
                ],
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

#[derive(Debug, Clone, Deserialize, Serialize, Queryable, Insertable, AsChangeset)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = self::schema::qc_forms)]
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

pub fn time_default() -> Time {
    Time(time::OffsetDateTime::now_utc())
}

#[allow(warnings)]
mod tests {

    mod test2 {
        use diesel::{sql_types::Bool, sqlite::Sqlite, BoxableExpression};

        use super::super::schema::qc_forms;

        type DynExpr =
            Box<dyn BoxableExpression<qc_forms::table, Sqlite, SqlType = diesel::sql_types::Bool>>;

        fn test3() {
            let _t: DynExpr = Box::new(diesel::dsl::sql::<Bool>("creation"));
            // diesel::sql_query(query)
        }
    }

    use std::collections::HashMap;

    use diesel::RunQueryDsl;
    use rocket::{http::Status, local::blocking::Client};
    use time::OffsetDateTime;

    use crate::{
        database::{create::NewQCForm, Time},
        qc_checklist::{QCChecklist, QuestionAnswer, QuestionAnswers},
    };

    use super::{schema::qc_forms, Db};

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
