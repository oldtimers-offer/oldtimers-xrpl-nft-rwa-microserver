use actix_web::{web, App, HttpServer};
use actix_web::http::header;
use actix_cors::Cors;
use dotenvy::dotenv;
use sqlx::mysql::MySqlPoolOptions;
use std::env;
use redis::Client as RedisClient;

use routes::nft::{get_vehicle_nft_metadata, mint_vehicle_nft};
use routes::widget::serve_dvp_widget;
use routes::qr::get_vehicle_qr;

mod db;
mod metadata;
mod xrpl;
mod redis_lock;

mod routes {
    pub mod nft;
    pub mod widget;
    pub mod qr;
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let database_url =
        env::var("DATABASE_URL").expect("DATABASE_URL must be set in environment");

    let redis_client = RedisClient::open(
        env::var("REDIS_URL").unwrap_or("redis://127.0.0.1/".to_string())
    )
    .expect("Invalid Redis URL");

    let redis_client = web::Data::new(redis_client);

    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to MySQL");

    let passport_image = env::var("NFT_PASSPORT_IMAGE")
        .unwrap_or_else(|_| "https://images.oldtimersoffer.com/nft/passport.png".to_string());

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("https://oldtimersoffer.com")
            .allowed_origin("https://app.oldtimersoffer.com")
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![
                header::AUTHORIZATION,
                header::ACCEPT,
                header::CONTENT_TYPE,
            ])
            .supports_credentials();

        App::new()
            .wrap(cors)
            .app_data(web::Data::new(pool.clone()))
            .app_data(redis_client.clone())
            .app_data(web::Data::new(passport_image.clone()))
            .service(get_vehicle_nft_metadata)
            .service(mint_vehicle_nft)
            .service(serve_dvp_widget)
            .service(get_vehicle_qr)
    })
    .bind(("0.0.0.0", 4000))?
    .run()
    .await
}