use rocket::response::Responder;

use rocket::serde::Serialize;

use rocket_sync_db_pools::diesel;

use super::search::compiler::ExpressionParserError;
use super::search::VisitorError;

#[derive(thiserror::Error, Debug, Serialize)]
pub enum DataBaseError {
    #[error("Tried to update finalized form")]
    UpdatedFinalized,
    #[error("{0}")]
    #[serde(serialize_with = "nothing")]
    DbError(#[from] diesel::result::Error),
    #[error("A form with the provided OEM serial alreadt exists")]
    ExistingOemSerial,
    #[error("A form with the provided ASM serial alreadt exists")]
    ExistingAsmSerial,
    #[error("A form with the provided Item serial alreadt exists")]
    ExistingItemSerial,
    #[error("An error occured when parsing database search query: {0}")]
    DataBaseSearchError(#[from] ExpressionParserError<VisitorError>),
    #[error("Invalid column specified '{0:?}'")]
    InvalidColumn(String),
}
fn nothing<T: std::fmt::Debug, S>(t: &T, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    s.serialize_str(&format!("{:?}", t))
}

impl<'r> Responder<'r, 'static> for DataBaseError {
    fn respond_to(
        self,
        _: &'r rocket::Request<'_>,
    ) -> std::result::Result<rocket::Response<'static>, rocket::http::Status> {
        use rocket::response::Response;
        use std::io::Cursor;

        Response::build()
            .header(rocket::http::ContentType::Plain)
            .status(rocket::http::Status::BadRequest)
            .streamed_body(Cursor::new(serde_json::to_vec(&self).unwrap_or(Vec::new())))
            .ok()
    }
}

pub type Result<T, E = DataBaseError> = std::result::Result<T, E>;
