#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_sync_db_pools;
use rocket::fs::{relative, FileServer};

mod diesel_sqlite;
pub mod schema;

mod manual {
    use rocket::fs::NamedFile;
    use std::path::{Path, PathBuf};
    #[get("/")]
    pub async fn index() -> Option<NamedFile> {
        let home_path = Path::new(super::relative!("static")).join("index.html");
        NamedFile::open(home_path).await.ok()
    }
    #[get("/fais")]
    pub async fn fais() -> Option<NamedFile> {
        let fais_path = Path::new(super::relative!("static")).join("fais.html");
        NamedFile::open(fais_path).await.ok()
    }
    #[get("/register")]
    pub async fn register() -> Option<NamedFile> {
        let fais_path = Path::new(super::relative!("static")).join("register.html");
        NamedFile::open(fais_path).await.ok()
    }
    #[get("/css/<path..>")]
    pub async fn css_file(path: PathBuf) -> Option<NamedFile> {
        let file_path = Path::new(super::relative!("static")).join(path);
        NamedFile::open(file_path).await.ok()
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![manual::index, manual::fais, manual::register])
        .mount("/", FileServer::from(relative!("static")))
        .mount("/css", routes![manual::css_file])
        .attach(diesel_sqlite::stage())
}
