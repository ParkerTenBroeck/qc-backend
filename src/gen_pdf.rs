// use headless_chrome::types::PrintToPdfOptions;
use rocket::fairing::AdHoc;
use rocket_dyn_templates::context;
use rocket_dyn_templates::Template;
use std::sync::OnceLock;

use crate::Config;
use crate::database;
use crate::database::Db;

type Result<T, E = rocket::response::Debug<diesel::result::Error>> = std::result::Result<T, E>;

#[get("/svg/<id>")]
pub async fn generate_svg(db: Db, id: i32) -> Result<Template> {
    let form = db.get_form(id).await?;

    Ok(Template::render(
        "pdf",
        context! {
            form: form
        },
    ))
}

#[get("/svg2/<id>/form.svg")]
pub async fn generate_svg2(db: Db, id: i32) -> Result<Template> {
    let form = db.get_form(id).await?;

    Ok(Template::render(
        "pdf2",
        context! {
            form: form,
        },
    ))
}

#[get("/printable/<id>")]
pub async fn printable(items: &Config, id: i32, db: Db) -> database::Result<Template> {
    let values = db.get_form(id).await?;

    Ok(Template::render(
        "printable",
        context! {
            items: &items.0,
            values,
            // filtered_questions,
        },
    ))
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("pdf_gen", |rocket| async {
        rocket.mount(
            "/",
            routes![printable, generate_svg, generate_svg2],
        )
    })
}
