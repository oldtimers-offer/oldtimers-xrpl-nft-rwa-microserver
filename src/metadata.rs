use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VehicleMetadata {
    pub schema: String,

    #[serde(rename = "type")]
    pub type_field: String,

    pub version: u32,
    pub name: String,
    pub description: String,
    pub image: String,
    pub external_url: String,
    pub identifiers: Identifiers,
    pub issuer: Issuer,
    pub attributes: Vec<Attribute>,
    pub vehicle: Vehicle,
    pub media: Media,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Identifiers {
    pub vehicle_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Issuer {
    pub name: String,
    pub website: String,
    pub app: Option<AppInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppInfo {
    pub android: Option<String>,
    pub windows: Option<String>,
    pub web: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Attribute {
    pub trait_type: String,
    pub value: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Vehicle {
    #[serde(rename = "type")]
    pub vehicle_type: String,
    pub make: String,
    pub model: String,
    pub year: u32,
    pub color: String,
    pub mileage: Mileage,
    pub country: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Mileage {
    pub value: u64,
    pub unit: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Media {
    pub primary_image: String,
}