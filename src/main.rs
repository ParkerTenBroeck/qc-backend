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
    use rocket_dyn_templates::handlebars::{Helper, Handlebars, Context, RenderContext, Output, HelperResult, handlebars_helper, RenderError};
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

    pub fn set_var(
        h: &Helper<'_, '_>,
        gb: &Handlebars<'_>,
        c: &Context,
        rc: &mut RenderContext<'_, '_>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let param = h
            .param(0)
            .ok_or(RenderError::new("Variable name non existant"))?
            .value()
            .as_str()
            .ok_or(RenderError::new("Expected string"))?;
       
            let param2 = h
            .param(1)
            .ok_or(RenderError::new("Variable name non existant"))?
            .value();

        
       
        let mut context = c.to_owned();
        // rc.context()
        if let Some(some) = context.data_mut().as_object_mut(){
            some.insert(param.to_string(), param2.to_owned());
        }

        rc.set_context(context);
        println!("{:#?}", c.data());
        Ok(())
    }

    // pub fn contains(
    //     h: &Helper<'_, '_>,
    //     _: &Handlebars<'_>,
    //     _: &Context,
    //     _rc: &mut RenderContext<'_, '_>,
    //     out: &mut dyn Output,
    // ) -> HelperResult {
    //     println!("{:#?}", c.data());
    //     Ok(())
    // }
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
                .register_helper("set_var", Box::new(helper::set_var));
            engine.handlebars
                .register_helper("contains", Box::new(helper::contains));
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
        .mount(
            "/",
            FileServer::new(
                "./RCT-FormBuilder",
                Options::DotFiles | Options::Index | Options::IndexFile,
            )
            .rank(10),
        )
}
