//! Models for (de)serialization.

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Chart {
    #[serde(rename = "chart_code")]
    pub chart_code: String,
    #[serde(rename = "chart_name")]
    pub chart_name: String,
    #[serde(rename = "pdf_name")]
    pub pdf_name: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Airport {
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "apt_ident")]
    pub apt_ident: Option<String>,
    #[serde(rename = "record")]
    pub records: Option<Vec<Chart>>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct City {
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "airport_name")]
    pub airports: Vec<Airport>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct State {
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "city_name")]
    pub city_names: Vec<City>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct DigitalTpp {
    #[serde(rename = "state_code")]
    pub state_codes: Vec<State>,
}
