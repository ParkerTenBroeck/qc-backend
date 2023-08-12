// use headless_chrome::types::PrintToPdfOptions;
use rocket::fairing::AdHoc;
use rocket_dyn_templates::context;
use rocket_dyn_templates::Template;

use crate::database;
use crate::database::Db;
use crate::Config;

#[get("/printable/<id>")]
pub async fn printable(items: &Config, id: i32, db: Db) -> database::Result<Template> {
    let values = db.get_form(id).await?;

    Ok(Template::render(
        "printable",
        context! {
            items: &items.0,
            values,
        },
    ))
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("pdf_gen", |rocket| async {
        rocket.mount("/", routes![printable])
    })
}
