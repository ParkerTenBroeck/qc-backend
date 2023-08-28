use crate::json_text::JsonText;

use rocket::response::status::Created;
use rocket::serde::{json::Json, Deserialize, Serialize};

use rocket_sync_db_pools::diesel;

use crate::qc_checklist::QCChecklist;

use crate::time::Time;

use self::diesel::prelude::*;

use super::*;

#[derive(Debug, Clone, Deserialize, Serialize, Queryable, Insertable, AsChangeset)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = super::schema::qc_forms)]
#[diesel(treat_none_as_null = true)]
pub struct NewQCForm {
    #[serde(skip_deserializing)]
    #[serde(default)]
    pub finalized: bool,
    // #[serde(skip_deserializing)]
    #[serde(default = "time_default")]
    pub creation_date: Time,
    // #[serde(skip_deserializing)]
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

#[post("/new_post", data = "<post>")]
pub(super) async fn new_post(
    db: Db,
    post: Json<NewQCForm>,
) -> Result<Created<Json<ExistingQCForm>>> {
    let post: Json<ExistingQCForm> = db
        .run(move |conn| {
            let count: i64 = qc_forms::table
                .filter(qc_forms::asm_serial.eq(&post.asm_serial))
                .count()
                .get_result(conn)?;
            if count > 0 {
                return Err(DataBaseError::ExistingAsmSerial);
            }
            let count: i64 = qc_forms::table
                .filter(qc_forms::item_serial.eq(&post.item_serial))
                .count()
                .get_result(conn)?;
            if count > 0 {
                return Err(DataBaseError::ExistingItemSerial);
            }

            let count: i64 = qc_forms::table
                .filter(qc_forms::oem_serial.eq(&post.oem_serial))
                .count()
                .get_result(conn)?;
            if count > 0 {
                return Err(DataBaseError::ExistingOemSerial);
            }

            diesel::insert_into(qc_forms::table)
                .values(&*post)
                .execute(conn)?;

            let res: ExistingQCForm = qc_forms::table.order(qc_forms::id.desc()).first(conn)?;

            Result::<Json<ExistingQCForm>>::Ok(res.into())
        })
        .await?;
    Ok(Created::new("/").body(post))
}
