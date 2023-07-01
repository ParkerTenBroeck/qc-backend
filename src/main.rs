use rocket::fs::{FileServer, Options};

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_sync_db_pools;

use rocket_dyn_templates::Template;

pub mod database;
pub mod qc_checklist;
pub mod qurry_builder;
pub mod schema;
pub mod time;

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(Template::custom(|_engins| {
            // engins
            //     .handlebars
            //     .register_helper("string_escape", Box::new(helpers::string_escape));
        }))
        // .attach(login_api::stage())
        // .attach(computer_api::stage())
        .attach(database::stage())
        .mount(
            "/",
            FileServer::new("./static/RCT-FormBuilder", Options::DotFiles),
        )
    // .mount("/", routes![index])
    // .mount(
    //     "/",
    //     routes![computers_page, monitors_page, parts_page, others_page],
    // )
    // .register("/", catchers![not_authorized])
}
