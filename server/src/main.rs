#[macro_use]
extern crate rocket;

mod catchers;
mod constants;
mod dtos;
mod environment;
mod guards;
mod routes;
mod utils;
mod websockets;

use crate::catchers::unauthorized;
use crate::environment::Env;
use crate::routes::chats::{create_chat, get_chat_by_id, get_chat_messages, get_chats};
use crate::routes::files::download_file;
use crate::routes::users::{get_user, get_users};
use crate::websockets::server::launch_websocket_server;
use dotenv::dotenv;
use rocket::catchers;
use rocket::routes;
use rocket::{Build, Rocket};
use rocket_cors::{AllowedOrigins, CorsOptions};
use routes::auth::{login, logout, register};
use sqlx::PgPool;
use tokio::task;

#[launch]
async fn rocket() -> Rocket<Build> {
    dotenv().ok();

    let database_url = Env::database_url();
    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to the database");

    let allowed_origins = AllowedOrigins::some_exact(&[Env::client_url()]);
    let cors = CorsOptions {
        allowed_origins,
        allowed_methods: vec!["POST", "GET", "OPTIONS", "PUT", "DELETE", "PATCH"]
            .into_iter()
            .map(|method| method.parse().unwrap())
            .collect(),
        allowed_headers: rocket_cors::AllowedHeaders::some(&["Authorization", "Content-Type"]),
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()
    .expect("Failed to create CORS");

    task::spawn(launch_websocket_server(pool.clone()));

    rocket::build()
        .manage(pool)
        .attach(cors)
        .mount(
            "/api/users",
            routes![register, login, logout, get_users, get_user],
        )
        .mount(
            "/api/chats",
            routes![create_chat, get_chats, get_chat_by_id, get_chat_messages],
        )
        .mount("/api/files", routes![download_file])
        .register("/", catchers![unauthorized])
}
