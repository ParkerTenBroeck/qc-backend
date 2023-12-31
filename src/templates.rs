use std::collections::HashMap;

use rocket::{fairing::AdHoc, http::Status};
use rocket_dyn_templates::{context, Template};
use serde_json::Value;

use crate::{
    database::{self, Db},
    Config,
};

//idk if this is needed but whatever
#[get("/qc_form", rank = 3)]
async fn qc_form(items: &Config) -> Template {
    Template::render(
        "qc_form",
        context! {
            items: &items.0,
        },
    )
}

#[get("/qc_form?<values..>", rank = 2)]
async fn qc_form_provided(
    items: &Config,
    values: HashMap<String, String>,
) -> Result<Template, Status> {
    let values: HashMap<String, Value> = values
        .into_iter()
        .map(|(key, value)| {
            (
                key,
                serde_json::de::from_str(&value).unwrap_or(Value::String(value)),
            )
        })
        .collect();

    Ok(Template::render(
        "qc_form",
        context! {
            items: &items.0,
            values
        },
    ))
}

#[get("/qc_form/<id>", rank = 1)]
async fn qc_form_id(items: &Config, id: i32, db: Db) -> database::Result<Template> {
    let values = db.get_form(id).await?;
    Ok(Template::render(
        "qc_form",
        context! {
            items: &items.0,
            values
        },
    ))
}

#[get("/database")]
async fn database_page(items: &Config) -> Template {
    Template::render(
        "database",
        context! {
            items: &items.0,
        },
    )
}

#[get("/printable/<id>?finalize")]
pub async fn printable_finaize(items: &Config, id: i32, db: Db) -> database::Result<Template> {
    use database::schema::qc_forms;
    use diesel::*;
    let values = db
        .run(move |conn| {
            diesel::update(qc_forms::table.find(id))
                .set(qc_forms::finalized.eq(true))
                .execute(conn)?;
            database::Result::<database::ExistingQCForm>::Ok(
                database::schema::qc_forms::table
                    .find(id)
                    .get_result::<database::ExistingQCForm>(conn)?,
            )
        })
        .await?;

    Ok(Template::render(
        "printable",
        context! {
            items: &items.0,
            values,
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
        },
    ))
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Templates", |rocket| async {
        rocket.mount(
            "/",
            routes![
                qc_form,
                qc_form_id,
                qc_form_provided,
                database_page,
                printable,
                printable_finaize
            ],
        )
    })
}
