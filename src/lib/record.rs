use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Deserialize)]
pub struct Record {
    pub r#type: String,
    pub client: u16,
    pub tx: u32,
    #[serde(deserialize_with = "csv::invalid_option")]
    pub amount: Option<f64>,
}