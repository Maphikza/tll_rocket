use crate::schema::users;
use rocket::fairing::AdHoc;
use rocket::response::{status::Created, Debug};
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::{Build, Rocket};

use rocket_sync_db_pools::diesel;

use self::diesel::prelude::*;

#[database("diesel")]

struct Db(diesel::SqliteConnection);

type Result<T, E = Debug<diesel::result::Error>> = std::result::Result<T, E>;

#[derive(Debug, Clone, Deserialize, Serialize, Queryable, Insertable)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = crate::schema::users)]
struct User {
    id: i32,
    username: String,
    email: String,
    password: String,
    admin: bool,
}

#[derive(Clone, Insertable)]
#[diesel(table_name = crate::schema::users)]
pub struct NewUser {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NewUserInput {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[post("/", data = "<user_input>")]
async fn register(db: Db, user_input: Json<NewUserInput>) -> Result<Created<Json<User>>> {
    let user = NewUser {
        username: user_input.username.clone(),
        email: user_input.email.clone(),
        password: user_input.password.clone(),
    };

    let user_value = user.clone();
    db.run(move |conn| {
        diesel::insert_into(users::table)
            .values(&user_value)
            .execute(conn)
    })
    .await?;

    // assuming users.order(id.desc()).first(conn) will fetch the newly created user
    let created_user: Result<User, diesel::result::Error> = db
        .run(move |conn| users::table.order(users::id.desc()).first::<User>(conn))
        .await;

    let created_user = created_user?;
    Ok(Created::new("/").body(Json(created_user)))
}

#[get("/<id>")]
async fn read(db: Db, id: i32) -> Option<Json<User>> {
    db.run(move |conn| users::table.filter(users::id.eq(id)).first(conn))
        .await
        .map(Json)
        .ok()
}

#[delete("/<id>")]
async fn delete(db: Db, id: i32) -> Result<Option<()>> {
    let affected = db
        .run(move |conn| {
            diesel::delete(users::table)
                .filter(users::id.eq(id))
                .execute(conn)
        })
        .await?;

    Ok((affected == 1).then(|| ()))
}

async fn run_migrations(rocket: Rocket<Build>) -> Rocket<Build> {
    use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

    const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

    Db::get_one(&rocket)
        .await
        .expect("database connection")
        .run(|conn| {
            conn.run_pending_migrations(MIGRATIONS)
                .expect("diesel migrations");
        })
        .await;

    rocket
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Diesel SQLite Stage", |rocket| async {
        rocket
            .attach(Db::fairing())
            .attach(AdHoc::on_ignite("Diesel Migrations", run_migrations))
            .mount("/diesel", routes![register, read, delete])
    })
}
