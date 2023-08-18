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

pub mod database;
pub mod json_text;
pub mod qc_checklist;
pub mod qurry_builder;
pub mod schema;
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

// rocket::log

#[launch]
fn rocket() -> _ {
    rocket::build()
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

            if path.exists() && path.is_file(){
                match Config::load_from_file(&path){
                    Ok(ok) => Ok(rocket.manage(ok)),
                    Err(err) => {
                        rocket::error!("Provided config '{}' is malformed\n{err:?}",path.display());
                        todo!()
                    }
                }
            }else{
                if path.exists(){
                    rocket::error!("Provided config path '{}' is not a file", path.display());
                }else{
                    rocket::error!("Provided config path '{}' does not exist", path.display());
                }
                Err(rocket)
            }
        }))
        .attach(AdHoc::try_on_ignite("Scripting", |rocket| async{
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

            if !path.exists(){
                rocket::error!("provided script_dir {} does not exist", path.display());
                return Err(rocket)
            }

            if path.is_file(){
                rocket::error!("provided script_dir {} is a file", path.display());
                return Err(rocket)    
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
                    if let Some(name) = path.path().file_stem().map(|s| s.to_str()).unwrap_or(None) {
                        // println!("{name}");
                        engine
                            .handlebars
                            .register_script_helper_file(name, path.path())?;
                    }
                }
                Ok(())
            })))
        }))
        // .attach()
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
