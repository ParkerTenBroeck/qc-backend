use rocket::{fairing::AdHoc, request::FromRequest};

#[derive(Debug, Clone, Eq, PartialEq)]
struct AdminPassword(pub String);

#[derive(Debug)]
pub struct Admin(std::marker::PhantomData<()>);

#[derive(Debug)]
pub struct InvalidPassword;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Admin {
    type Error = InvalidPassword;

    async fn from_request(
        req: &'r rocket::Request<'_>,
    ) -> rocket::request::Outcome<Self, Self::Error> {
        let admin_password = req.rocket().state::<AdminPassword>();
        let provided_password = req.headers().get_one("ADMIN_PASSWORD");

        if let (Some(admin_password), Some(provided_password)) = (admin_password, provided_password)
        {
            if admin_password.0 == provided_password {
                rocket::request::Outcome::Success(Admin(std::marker::PhantomData))
            } else {
                rocket::request::Outcome::Error((
                    rocket::http::Status::Forbidden,
                    InvalidPassword,
                ))
            }
        } else {
            rocket::request::Outcome::Error((rocket::http::Status::Forbidden, InvalidPassword))
        }
    }
}

pub fn stage() -> AdHoc {
    let pwd = if let Ok(env) = std::env::var("ADMIN_PWD") {
        env
    } else {
        rocket::warn!("Failed to load ADMIN_PWD from env.. using default password");
        "enterprise".into()
    };
    let pwd = AdminPassword(pwd);
    AdHoc::on_ignite("AdminPassword", |rocket| async { rocket.manage(pwd) })
}
