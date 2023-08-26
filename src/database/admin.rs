use crate::admin_pwd::Admin;

use rocket::serde::json::Json;

use rocket_sync_db_pools::diesel;

use self::diesel::prelude::*;

use super::*;

#[delete("/delete_post/<id>")]
pub(super) async fn delete_post(db: Db, id: i32, _admin: Admin) -> Result<()> {
    db.run(move |conn| {
        diesel::delete(qc_forms::table.find(id)).execute(conn)?;
        Ok(())
    })
    .await
}

#[post("/definalize_post/<id>")]
pub(super) async fn definalize_post(
    db: Db,
    id: i32,
    _admin: Admin,
) -> Result<Json<ExistingQCForm>> {
    db.run(move |conn| {
        diesel::update(qc_forms::table.find(id))
            .set(qc_forms::finalized.eq(false))
            .execute(conn)?;
        Ok(qc_forms::table
            .find(id)
            .get_result::<ExistingQCForm>(conn)?
            .into())
    })
    .await
}
