use std::path::Path;

use rocket::{
    fairing::AdHoc,
    figment::value::magic::RelativePathBuf,
    fs::{FileServer, Options},
    request::FromRequest,
};

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_sync_db_pools;

use rocket_dyn_templates::Template;

pub mod admin_pwd;
pub mod database;
pub mod json_text;
pub mod qc_checklist;
pub mod snapshots;
pub mod templates;
pub mod time;

pub mod copy_session;

#[derive(Debug)]
pub struct Config(pub serde_json::Value);

impl Config {
    fn load_from_file(path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = std::fs::read_to_string(path)?;
        Ok(Self(json5::from_str(&contents)?))
    }
}

#[derive(Debug)]
pub struct FailedToObtainConfig;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for &'r Config {
    type Error = FailedToObtainConfig;

    async fn from_request(
        request: &'r rocket::Request<'_>,
    ) -> rocket::request::Outcome<Self, Self::Error> {
        if let Some(config) = request.rocket().state::<Config>() {
            rocket::request::Outcome::Success(config)
        } else {
            rocket::outcome::Outcome::Failure((
                rocket::http::Status::InternalServerError,
                FailedToObtainConfig,
            ))
        }
    }
}

mod helper {
    use rocket_dyn_templates::handlebars::{
        handlebars_helper, Context, Handlebars, Helper, HelperResult, Output, RenderContext,
    };
    use serde_json::Value;

    pub fn json_stringify(
        h: &Helper<'_, '_>,
        _: &Handlebars<'_>,
        _: &Context,
        _rc: &mut RenderContext<'_, '_>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let json = h
            .param(0)
            .and_then(|v| serde_json::to_string(v.value()).ok())
            .unwrap_or("{}".to_owned());
        out.write(&json)?;
        Ok(())
    }

    handlebars_helper!(contains: |ar: Value, val: Value|{
            ar.as_array().map(|v|v.contains(&val)).unwrap_or_else(||ar.eq(&val))
    });
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(admin_pwd::stage())
        .attach(AdHoc::try_on_ignite("Config", |rocket| async {
            let path = rocket
                .figment()
                .extract_inner::<RelativePathBuf>("config")
                .map(|p| p.relative());

            let path = match path {
                Ok(dir) => dir,
                Err(e) => {
                    rocket::config::pretty_print_error(e);
                    return Err(rocket);
                }
            };

            if path.exists() && path.is_file() {
                match Config::load_from_file(&path) {
                    Ok(ok) => Ok(rocket.manage(ok)),
                    Err(err) => {
                        rocket::error!(
                            "Provided config '{}' is malformed\n{err:?}",
                            path.display()
                        );
                        todo!()
                    }
                }
            } else {
                if path.exists() {
                    rocket::error!("Provided config path '{}' is not a file", path.display());
                } else {
                    rocket::error!("Provided config path '{}' does not exist", path.display());
                }
                Err(rocket)
            }
        }))
        .attach(AdHoc::try_on_ignite("Scripting", |rocket| async {
            let path = rocket
                .figment()
                .extract_inner::<RelativePathBuf>("script_dir")
                .map(|p| p.relative());

            let path = match path {
                Ok(dir) => dir,
                Err(e) => {
                    rocket::config::pretty_print_error(e);
                    return Err(rocket);
                }
            };

            if !path.exists() {
                rocket::error!("provided script_dir {} does not exist", path.display());
                return Err(rocket);
            }

            if path.is_file() {
                rocket::error!("provided script_dir {} is a file", path.display());
                return Err(rocket);
            }

            Ok(rocket.attach(Template::try_custom(move |engine| {
                engine
                    .handlebars
                    .register_helper("json_stringify", Box::new(helper::json_stringify));
                engine
                    .handlebars
                    .register_helper("contains", Box::new(helper::contains));
                engine.handlebars.set_strict_mode(true);

                let scripts = std::fs::read_dir(&path)?;
                for path in scripts.flatten() {
                    // path.path().file_stem()
                    if let Some(name) = path.path().file_stem().map(|s| s.to_str()).unwrap_or(None)
                    {
                        // println!("{name}");
                        engine
                            .handlebars
                            .register_script_helper_file(name, path.path())?;
                    }
                }
                Ok(())
            })))
        }))
        .attach(snapshots::stage())
        .attach(database::stage())
        .attach(copy_session::stage())
        .attach(templates::stage())
        .mount(
            "/",
            FileServer::new(
                "./static",
                Options::DotFiles | Options::Index | Options::IndexFile,
            )
            .rank(5),
        )
}

mod test {
    use diesel::{
        expression::AsExpression, query_dsl::methods::FilterDsl, serialize::ToSql, sql_types::Text,
        sqlite::Sqlite, BoolExpressionMethods, ExpressionMethods, IntoSql,
    };
    use serde_json::Value;

    use crate::database::schema::qc_forms;

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

    // impl ToSql<diesel::sql_types::Text, Sqlite> for Stupid {
    //     fn to_sql<'b>(&'b self, out: &mut diesel::serialize::Output<'b, '_, Sqlite>) -> diesel::serialize::Result {
    //         // Bound::new(12);
    //         match &self.0{
    //             Value::Null => return Ok(diesel::serialize::IsNull::Yes),
    //             Value::Bool(bool) => out.set_value(if *bool {1} else {0}),
    //             Value::Number(num) => {
    //                 if let Some(uint) = num.as_u64(){
    //                     out.set_value(uint as i64)
    //                 }else if let Some(int) = num.as_i64(){
    //                     out.set_value(int)
    //                 }else if let Some(float) = num.as_f64(){
    //                     out.set_value(float)
    //                 }else{
    //                     let arb = num.to_string();
    //                     out.set_value(arb);
    //                 }
    //             },
    //             Value::String(str) => {
    //                 out.set_value(str.as_str());
    //             },
    //             other => {
    //                 out.set_value(serde_json::to_string(&other).unwrap_or(String::new()))
    //             }
    //         }
    //         Ok(diesel::serialize::IsNull::No)
    //     }
    // }

    // fn bind_val<ST>(post_sql: &str, value: Value, post_fix: &str) -> DynExpr{
    //     // 12.to_sql(out)
    //     use diesel::sql_types::*;
    //     match value{
    //         Value::Null => {
    //             Box::new(diesel::dsl::sql::<Bool>(post_sql).is_null())
    //         },
    //         Value::Bool(val) => {
    //             Box::new(diesel::dsl::sql::<Bool>(post_sql).bind::<Bool, _>(val))
    //         },
    //         Value::Number(num) => {
    //             if let Some(uint) = num.as_u64(){
    //                 Box::new(diesel::dsl::sql::<Bool>(post_sql).bind::<BigInt, _>(uint as i64))
    //             }else if let Some(int) = num.as_i64(){
    //                 Box::new(diesel::dsl::sql::<Bool>(post_sql).bind::<BigInt, _>(int))
    //             }else if let Some(float) = num.as_f64(){
    //                 Box::new(diesel::dsl::sql::<Bool>(post_sql).bind::<Double, _>(float))
    //             }else{
    //                 let arb = num.to_string();
    //                 Box::new(diesel::dsl::sql::<Bool>(post_sql).bind::<Text, _>(arb))
    //             }
    //         },
    //         Value::String(str) => {
    //             Box::new(diesel::dsl::sql::<Bool>(post_sql).bind::<Text, _>(str))
    //         },
    //         other => {
    //             Box::new(diesel::dsl::sql::<Bool>(post_sql).bind::<Text, _>(serde_json::to_string(&other).unwrap_or(String::new())))
    //         }
    //     }
    // }

    #[test]
    fn bruh() {
        use diesel::sql_types::*;
        let sql1 = diesel::dsl::sql::<Bool>("ifnull(")
            .sql("asm_serial = ")
            .sql(&to_sql_str(&Value::Bool(false)))
            .sql(", FALSE)");
        let sql2 = diesel::dsl::sql::<Bool>("ifnull(")
            .sql("asm_serial = ")
            .sql(&to_sql_str(&Value::String("\"'".into())))
            .sql(", FALSE)");
        let sql3 = diesel::dsl::sql::<Bool>("ifnull(")
            .sql("asm_serial = ")
            .sql(&to_sql_str(&Value::Null))
            .sql(", FALSE)");
        let sql4 = diesel::dsl::sql::<Bool>("ifnull(")
            .sql("asm_serial = ")
            .sql(&to_sql_str(&Value::Array(vec![
                Value::Bool(false),
                Value::Bool(true),
                Value::Null,
                Value::String("\"' Hii~".into()),
                Value::Number(12.into()),
            ])))
            .sql(", FALSE)");
        let sql = sql1.or(sql2).or(sql3).or(sql4);

        println!(
            "{}",
            diesel::debug_query::<Sqlite, _>(&qc_forms::table.filter(sql))
        );
    }
}
