use actix_web::{web, get, Responder, HttpResponse};
use sqlx::MySqlPool;
use qrcode::{QrCode, EcLevel};
use image::Luma;
use base64::{Engine as _, engine::general_purpose};
use std::io::Cursor;
use crate::db::find_vehicle_nft;
 
#[get("/nft/vehicle/{id}/qr")]
pub async fn get_vehicle_qr(
    path: web::Path<u64>,
    pool: web::Data<MySqlPool>,
) -> impl Responder {
    let vehicle_id = path.into_inner();
 
    let nft = match find_vehicle_nft(&pool, vehicle_id).await {
        Ok(Some(n)) => n,
        Ok(None) => return HttpResponse::NotFound().json(serde_json::json!({
            "error": "No NFT for this vehicle"
        })),
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    };
 
    let explorer_url = if nft.network == "testnet" {
        format!("https://testnet.xrpl.org/nft/{}", nft.nft_id)
    } else {
        format!("https://livenet.xrpl.org/nft/{}", nft.nft_id)
    };
 
    let code = match QrCode::with_error_correction_level(&explorer_url, EcLevel::M) {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("QR generation failed: {}", e)
        })),
    };
 
    let image = code.render::<Luma<u8>>()
        .min_dimensions(160, 160)
        .quiet_zone(true)
        .build();
 
    let mut png_bytes: Vec<u8> = Vec::new();
    let mut cursor = Cursor::new(&mut png_bytes);
    if let Err(e) = image.write_to(&mut cursor, image::ImageFormat::Png) {
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("PNG encoding failed: {}", e)
        }));
    }
 
    let b64 = general_purpose::STANDARD.encode(&png_bytes);
    let data_url = format!("data:image/png;base64,{}", b64);
 
    HttpResponse::Ok()
        .insert_header(("Cache-Control", "public, max-age=86400"))
        .insert_header(("Access-Control-Allow-Origin", "*"))
        .json(serde_json::json!({
            "vehicle_id": vehicle_id,
            "nft_id": nft.nft_id,
            "tx_hash": nft.tx_hash,
            "explorer_url": explorer_url,
            "qr_data_url": data_url
        }))
}