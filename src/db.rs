use sqlx::{FromRow, MySqlPool};

#[derive(Debug, Clone, FromRow)]
pub struct VehicleRow {
    pub vehicle_id: u64,
    pub model_year: Option<u64>,
    pub make_display: Option<String>,
    pub model_name: Option<String>,
    pub vehicle_type: Option<String>,
    pub odometer: Option<i32>,
    pub color: Option<String>,
    pub country_location: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
pub struct VehiclePhotoRow {
    pub original_url: String,
    pub thumbnail_url: Option<String>,
    pub medium_url: Option<String>,
    pub mobile_url: Option<String>,
    pub medium_large_url: Option<String>,
    pub large_url: Option<String>,
}

#[derive(Debug, FromRow)]
#[allow(dead_code)]
pub struct VehicleNftRow {
    pub vehicle_id: u64,
    pub nft_id: String,
}

pub async fn fetch_vehicle_by_id(
    pool: &MySqlPool,
    vehicle_id: u64,
) -> Result<Option<VehicleRow>, sqlx::Error> {
    let row = sqlx::query_as::<_, VehicleRow>(
        r#"
        SELECT
            vehicle_id,
            model_year,
            make_display,
            model_name,
            vehicle_description,
            vehicle_type,
            odometer,
            color,
            country_location
        FROM wp_vehicle_details
        WHERE vehicle_id = ?
        LIMIT 1
        "#,
    )
    .bind(vehicle_id)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

pub async fn fetch_primary_photo_by_vehicle_id(
    pool: &MySqlPool,
    vehicle_id: u64,
) -> Result<Option<VehiclePhotoRow>, sqlx::Error> {
    let row = sqlx::query_as::<_, VehiclePhotoRow>(
        r#"
        SELECT
            original_url,
            thumbnail_url,
            medium_url,
            mobile_url,
            medium_large_url,
            large_url
        FROM wp_vehicle_photos
        WHERE vehicle_id = ?
        ORDER BY sort_order ASC, id ASC
        LIMIT 1
        "#,
    )
    .bind(vehicle_id)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

pub async fn find_vehicle_nft(
    pool: &MySqlPool,
    vehicle_id: u64,
) -> Result<Option<VehicleNftRow>, sqlx::Error> {
    let row = sqlx::query_as::<_, VehicleNftRow>(
        r#"
        SELECT vehicle_id, nft_id
        FROM wp_vehicle_nfts
        WHERE vehicle_id = ?
        LIMIT 1
        "#,
    )
    .bind(vehicle_id)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

pub async fn insert_vehicle_nft(
    pool: &MySqlPool,
    vehicle_id: u64,
    nft_id: &str,
    tx_hash: Option<&str>,
    metadata_url: &str,
    network: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO wp_vehicle_nfts
        (vehicle_id, nft_id, tx_hash, metadata_url, network, status)
        VALUES (?, ?, ?, ?, ?, 'minted')
        "#,
    )
    .bind(vehicle_id)
    .bind(nft_id)
    .bind(tx_hash)
    .bind(metadata_url)
    .bind(network)
    .execute(pool)
    .await?;

    Ok(())
}