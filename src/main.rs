use actix_web::{web, App, HttpServer};
 use actix_web::http::header;
use actix_cors::Cors;
use dotenvy::dotenv;
use sqlx::mysql::MySqlPoolOptions;
use std::env;

use routes::nft::{get_vehicle_nft_metadata, mint_vehicle_nft};

mod db;
mod metadata;
mod xrpl;

mod routes {
    pub mod nft;
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let database_url =
        env::var("DATABASE_URL").expect("DATABASE_URL must be set in environment");

    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to MySQL");

    let passport_image = env::var("NFT_PASSPORT_IMAGE")
        .unwrap_or_else(|_| "https://images.oldtimersoffer.com/nft/passport.png".to_string());

    HttpServer::new(move || {
        let _cors = Cors::default()
            .allowed_origin("http://oldtimersoffer.com")
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![
                header::AUTHORIZATION,
                header::ACCEPT,
                header::CONTENT_TYPE,
            ])
            .supports_credentials();

        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(passport_image.clone()))
            .service(get_vehicle_nft_metadata)
            .service(mint_vehicle_nft)
    })
    .bind(("0.0.0.0", 4000))?
    .run()
    .await
}