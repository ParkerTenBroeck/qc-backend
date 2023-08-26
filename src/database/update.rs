use crate::json_text::JsonText;

use rocket::response::status::Accepted;

use rocket::serde::{json::Json, Deserialize, Serialize};

use rocket_sync_db_pools::diesel;

use crate::qc_checklist::QCChecklist;

use crate::time::Time;

use self::diesel::prelude::*;

use super::*;

#[derive(Debug, Default, Clone, Deserialize, Serialize, AsChangeset)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = super::schema::qc_forms)]
#[serde(default)]
pub struct QCFormUpdate {
    #[serde(skip_deserializing)]
    pub last_updated: Option<Time>,
    pub build_location: Option<String>,
    pub build_type: Option<String>,
    pub drive_type: Option<String>,
    pub item_serial: Option<String>,
    #[serde(deserialize_with = "deserialize_optional_field")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asm_serial: Option<Option<String>>,
    pub oem_serial: Option<String>,
    pub make_model: Option<String>,
    pub mso_installed: Option<bool>,
    pub operating_system: Option<String>,
    pub processor_gen: Option<String>,
    pub processor_type: Option<String>,
    pub qc_answers: Option<QCChecklist>,
    pub qc1_initial: Option<String>,
    #[serde(deserialize_with = "deserialize_optional_field")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qc2_initial: Option<Option<String>>,

    pub ram_size: Option<String>,
    pub ram_type: Option<String>,

    #[serde(deserialize_with = "deserialize_optional_field")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sales_order: Option<Option<String>>,
    pub drive_size: Option<String>,
    pub tech_notes: Option<String>,

    #[serde(deserialize_with = "deserialize_optional_field")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Option<JsonText>>,
}

fn deserialize_optional_field<'de, T, D>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    Ok(Some(Option::deserialize(deserializer)?))
}

#[post("/update_post/<id>", data = "<update>")]
pub(super) async fn update_post(
    db: Db,
    id: i32,
    mut update: Json<QCFormUpdate>,
) -> Result<Accepted<Json<ExistingQCForm>>> {
    update.last_updated = Some(time_default());
    let res: ExistingQCForm = db
        .run(move |conn| {
            let finalized: bool = qc_forms::table
                .find(id)
                .select(qc_forms::finalized)
                .get_result(conn)?;

            if finalized {
                return Err(DataBaseError::UpdatedFinalized);
            }

            diesel::update(qc_forms::table.filter(qc_forms::id.eq(id)))
                .set(&*update)
                .execute(conn)?;
            Ok(qc_forms::table.filter(qc_forms::id.eq(id)).first(conn)?)
        })
        .await?;
    Ok(Accepted(Json(res)))
}

#[post("/finalize_post/<id>")]
pub(super) async fn finalize_post(db: Db, id: i32) -> Result<Json<ExistingQCForm>> {
    db.run(move |conn| {
        diesel::update(qc_forms::table.find(id))
            .set(qc_forms::finalized.eq(true))
            .execute(conn)?;
        Ok(qc_forms::table
            .find(id)
            .get_result::<ExistingQCForm>(conn)?
            .into())
    })
    .await
}
