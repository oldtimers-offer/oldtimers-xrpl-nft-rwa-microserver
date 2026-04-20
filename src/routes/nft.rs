use actix_web::{get, post, web, HttpResponse, Responder, HttpRequest};
use sqlx::MySqlPool;
use std::env;

use crate::db::{fetch_primary_photo_by_vehicle_id, fetch_vehicle_by_id, VehiclePhotoRow, VehicleRow};
use crate::metadata::{
    Attribute, Identifiers, Issuer, AppInfo,Media, Mileage, Vehicle, VehicleMetadata,
};
use crate::xrpl::VehiclePassportMinter;

use crate::db::{find_vehicle_nft, insert_vehicle_nft};

fn slugify(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut last_dash = false;

    for ch in input.chars() {
        let c = ch.to_ascii_lowercase();

        if c.is_ascii_alphanumeric() {
            out.push(c);
            last_dash = false;
        } else if !last_dash {
            out.push('-');
            last_dash = true;
        }
    }

    out.trim_matches('-').to_string()
}

fn best_photo_url(photo: Option<&VehiclePhotoRow>) -> String {
    match photo {
        Some(p) => p
            .large_url
            .clone()
            .or_else(|| p.medium_large_url.clone())
            .or_else(|| p.medium_url.clone())
            .or_else(|| p.mobile_url.clone())
            .or_else(|| p.thumbnail_url.clone())
            .unwrap_or_else(|| p.original_url.clone()),
        None => "https://app.oldtimersoffer.com/nft/passport.png".to_string(),
    }
}

fn build_vehicle_metadata(vehicle: &VehicleRow, photo: Option<&VehiclePhotoRow>, passport_image: &str) -> VehicleMetadata {
    let year = vehicle.model_year.unwrap_or(0).max(0) as u32;
    let make = vehicle
        .make_display
        .clone()
        .unwrap_or_else(|| "Unknown".to_string());
    let model = vehicle
        .model_name
        .clone()
        .unwrap_or_else(|| "Vehicle".to_string());
    let vehicle_type = vehicle
        .vehicle_type
        .clone()
        .unwrap_or_else(|| "Vehicle".to_string());
    let color = vehicle
        .color
        .clone()
        .unwrap_or_else(|| "Unknown".to_string());
    let country = vehicle
        .country_location
        .clone()
        .unwrap_or_else(|| "Unknown".to_string());
    let mileage_value = vehicle.odometer.unwrap_or(0).max(0) as u64;

    let display_name = if year > 0 {
        format!("{year} {make} {model}")
    } else {
        format!("{make} {model}")
    };

    let slug = if year > 0 {
        slugify(&format!("{year} {make} {model}"))
    } else {
        slugify(&format!("{make} {model}"))
    };

    let primary_image = best_photo_url(photo);

    VehicleMetadata {
        schema: "oldtimersoffer-vehicle-metadata-v1".to_string(),
        type_field: "vehicle-passport".to_string(),
        version: 1,
        name: display_name,
        description: "Vehicle Passport NFT issued by Oldtimers Offer. \
                      Represents a verified vehicle listing anchored on the XRPL as a Real-World Asset (RWA). \
                      Acts as a persistent digital identity, enabling transparent and verifiable vehicle data."
                      .to_string(),
        image: passport_image.to_string(),
        external_url: format!(
            "https://app.oldtimersoffer.com/vehicle/classic-{}/{}",
            vehicle.vehicle_id, slug
        ),
        identifiers: Identifiers {
            vehicle_id: vehicle.vehicle_id.to_string(),
        },
        issuer: Issuer { 
            name: "Oldtimers Offer".to_string(),
            website: "https://oldtimersoffer.com".to_string(),
            app: Some(AppInfo {
                android: Some("https://play.google.com/store/apps/details?id=com.oldtimersoffer.app".to_string()),
                windows: Some("https://apps.microsoft.com/detail/9p8ltw7dv2f7".to_string()),
                web: Some("https://app.oldtimersoffer.com".to_string()),
            }),
        },
        attributes: vec![
            Attribute {
                trait_type: "Vehicle Type".to_string(),
                value: serde_json::json!(vehicle_type),
            },
            Attribute {
                trait_type: "Make".to_string(),
                value: serde_json::json!(make),
            },
            Attribute {
                trait_type: "Model".to_string(),
                value: serde_json::json!(model),
            },
            Attribute {
                trait_type: "Year".to_string(),
                value: serde_json::json!(year),
            },
            Attribute {
                trait_type: "Mileage".to_string(),
                value: serde_json::json!(format!("{mileage_value} miles")),
            },
            Attribute {
                trait_type: "Country".to_string(),
                value: serde_json::json!(country),
            },
        ],
        vehicle: Vehicle {
            vehicle_type: vehicle_type.to_lowercase(),
            make,
            model,
            year,
            color,
            mileage: Mileage {
                value: mileage_value,
                unit: "miles".to_string(),
            },
            country,
        },
        media: Media { primary_image },
    }
}

#[get("/nft/vehicle/{id}")]
pub async fn get_vehicle_nft_metadata(
    path: web::Path<u64>,
    pool: web::Data<MySqlPool>,
    passport_image: web::Data<String>,
) -> impl Responder {
    let vehicle_id = path.into_inner();

    let vehicle = match fetch_vehicle_by_id(&pool, vehicle_id).await {
        Ok(Some(v)) => v,
        Ok(None) => return HttpResponse::NotFound().json(serde_json::json!({
            "error": "Vehicle not found"
        })),
        Err(err) => return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Database error", "details": err.to_string()
        })),
    };

    let photo = match fetch_primary_photo_by_vehicle_id(&pool, vehicle_id).await {
        Ok(photo) => photo,
        Err(err) => return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Database error", "details": err.to_string()
        })),
    };

    let metadata = build_vehicle_metadata(&vehicle, photo.as_ref(), &passport_image);

    HttpResponse::Ok().json(metadata)
}

#[post("/nft/vehicle/{id}/mint")]
pub async fn mint_vehicle_nft(
    req: HttpRequest,
    path: web::Path<u64>,
    pool: web::Data<MySqlPool>,
) -> impl Responder {

    let expected_key = env::var("XRPL_API_KEY").unwrap_or_default();

    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok());

    let is_valid = match auth_header {
        Some(value) => value == format!("Bearer {}", expected_key),
        None => false,
    };

    if !is_valid {
        return HttpResponse::Unauthorized().json(serde_json::json!({
            "success": false,
            "error": "Unauthorized"
        }));
    }

    let vehicle_id = path.into_inner();

    let vehicle = match fetch_vehicle_by_id(&pool, vehicle_id).await {
        Ok(Some(v)) => v,
        Ok(None) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": "Vehicle not found"
            }));
        }
        Err(err) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Database error",
                "details": err.to_string()
            }));
        }
    };

    let seed = match env::var("XRPL_SEED") {
        Ok(v) => v,
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "XRPL_SEED is not set"
            }));
        }
    };

    let node_url = match env::var("XRPL_NODE_URL") {
        Ok(v) => v,
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "XRPL_NODE_URL is not set"
            }));
        }
    };

    let base_url = match env::var("XRPL_BASE_URL") {
        Ok(v) => v,
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "XRPL_BASE_URL is not set"
            }));
        }
    };

    let uri = format!("{}/nft/vehicle/{}", base_url.trim_end_matches('/'), vehicle.vehicle_id);

    let minter = VehiclePassportMinter::new();

    match find_vehicle_nft(&pool, vehicle_id).await {
        Ok(Some(existing)) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "error": "Vehicle already has NFT",
                "nft_id": existing.nft_id
            }));
        }
        Ok(None) => {}
        Err(err) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "error": "Database error while checking existing NFT",
                "details": err.to_string()
            }));
        }
    }

    match minter.mint_from_uri(&uri, &seed, &node_url).await {
        Ok(result) => {
            let metadata_url = uri.clone();

            if let Err(err) = insert_vehicle_nft(
                &pool,
                vehicle_id,
                &result.nft_id,
                result.tx_hash.as_deref(),
                &metadata_url,
                "mainnet",
            ).await {
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "success": false,
                    "error": format!("NFT minted but DB insert failed: {}", err),
                    "nft_id": result.nft_id,
                    "tx_hash": result.tx_hash,
                    "uri": result.uri
                }));
            }

            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "vehicle_id": vehicle_id,
                "nft_id": result.nft_id,
                "tx_hash": result.tx_hash,
                "uri": result.uri
            }))
        }
        Err(err) => {
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "error": err.to_string()
            }))
        }
    }
}