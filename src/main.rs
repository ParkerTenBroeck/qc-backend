use std::path::Path;

use rocket::{
    fs::{FileServer, Options},
    request::FromRequest,
};

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_sync_db_pools;

use rocket_dyn_templates::Template;

pub mod database;
pub mod gen_pdf;
pub mod qc_checklist;
pub mod qurry_builder;
pub mod schema;
pub mod templates;
pub mod time;
pub mod json_text;

pub mod copy_session;

#[derive(Debug)]
pub struct Config(pub serde_json::Value);

impl Config {
    fn load_from_file(path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = std::fs::read_to_string(path)?;
        Ok(Self(serde_json::from_str(&contents)?))
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
    use rocket_dyn_templates::handlebars::{Helper, Handlebars, Context, RenderContext, Output, HelperResult, handlebars_helper};
    use serde_json::Value;

    pub fn json_stringify(
        h: &Helper<'_, '_>,
        _: &Handlebars<'_>,
        _: &Context,
        _rc: &mut RenderContext<'_, '_>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let json = h.param(0).and_then(|v|serde_json::to_string(v.value()).ok()).unwrap_or("{}".to_owned());
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
        .manage(
            Config::load_from_file("./res/everything.json")
                .expect("Failed to load config file. Fatial Error"),
        )
        .attach(Template::custom(|engine| {
            engine.handlebars
                .register_helper("json_stringify", Box::new(helper::json_stringify));
            engine.handlebars
                .register_helper("contains", Box::new(helper::contains));
            engine.handlebars.set_strict_mode(true);

            let scripts = std::fs::read_dir("./template_scripts").unwrap();
            for path in scripts.flatten(){
                // path.path().file_stem()
                if let Some(name) = path.path().file_stem().map(|s|s.to_str()).unwrap_or(None){
                    println!("{name}");
                    engine.handlebars.register_script_helper_file(name, path.path()).unwrap();
                }
            }
            // engine.handlebars.register_script_helper("name", "script")?;
        }))
        .attach(database::stage())
        .attach(copy_session::stage())
        .attach(gen_pdf::stage())
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