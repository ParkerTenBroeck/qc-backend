use rocket::fairing::AdHoc;

pub fn stage() -> AdHoc {
    AdHoc::on_liftoff("Snapshots", |rocket| {
        Box::pin(async move {
            let shutdown = rocket.shutdown();

            // since we dont do anything yet just return early
            if true{
                return;
            }

            rocket::tokio::spawn(async move {
                let mut interval =
                    rocket::tokio::time::interval(rocket::tokio::time::Duration::from_secs(10));
                loop {
                    let shutdown = shutdown.clone();
                    rocket::tokio::pin!(shutdown);

                    rocket::tokio::select! {
                        _ = shutdown => return,
                        _ = interval.tick() => {

                            // println!("Do something here!!!");
                        }
                    }
                }
            });
        })
    })
}
