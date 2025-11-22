use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct LegacyResult {
    #[serde(rename = "k")]
    pub keyword: Option<String>,
    #[serde(rename = "pron")]
    pub pronounce: Option<HashMap<String, String>>,
    #[serde(rename = "para")]
    pub paraphrase: Option<Vec<String>>,
    #[serde(rename = "eg")]
    pub examples: Option<HashMap<String, Vec<Vec<String>>>>,
    #[serde(rename = "co")]
    pub collins: Option<CollinsData>,
}

#[derive(Debug, Deserialize)]
pub struct CollinsData {
    #[serde(rename = "li")]
    pub items: Option<Vec<CollinsItem>>,
    #[serde(rename = "star")]
    #[allow(dead_code)]
    pub star: Option<i32>,
    #[serde(rename = "rank")]
    pub rank: Option<String>,
    #[serde(rename = "pat")]
    #[allow(dead_code)]
    pub additional_pattern: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CollinsItem {
    #[serde(rename = "a")]
    #[allow(dead_code)]
    pub additional: Option<String>,
    #[serde(rename = "maj")]
    #[allow(dead_code)]
    pub major_trans: Option<String>,
    #[serde(rename = "eg")]
    pub examples: Option<Vec<Vec<String>>>,
}
