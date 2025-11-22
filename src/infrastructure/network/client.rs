use crate::domain::error::KdError;
use crate::domain::model::{OnlineSource, QueryResult};
use crate::domain::traits::Translator;
use crate::infrastructure::config::Config;
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

// Youdao API Response structures
#[derive(Deserialize, Debug)]
struct YoudaoResponse {
    #[serde(rename = "translation")]
    translations: Option<Vec<String>>,
    #[serde(rename = "basic")]
    basic: Option<BasicInfo>,
    #[serde(rename = "web")]
    web_translations: Option<Vec<WebTranslation>>,
    #[serde(rename = "errorCode")]
    error_code: String,
}

#[derive(Deserialize, Debug)]
struct BasicInfo {
    #[serde(rename = "phonetic")]
    pronunciation: Option<String>,
    explains: Option<Vec<String>>,
}

#[derive(Deserialize, Debug)]
struct WebTranslation {
    key: String,
    value: Vec<String>,
}

/// Youdao translator implementation
///
/// Reserved for future use with Translator trait abstraction.
#[allow(dead_code)]
pub struct YoudaoTranslator {
    client: Client,
    config: Config,
}

impl YoudaoTranslator {
    #[allow(dead_code)]
    pub fn new(client: Client, config: Config) -> Self {
        Self { client, config }
    }
}

#[async_trait]
impl Translator for YoudaoTranslator {
    async fn translate(&self, query: &str) -> Result<QueryResult, KdError> {
        query_youdao_impl(&self.client, &self.config, query).await
    }
}

// Public function for backward compatibility
pub async fn query_youdao(
    client: &Client,
    config: &Config,
    query: &str,
) -> Result<QueryResult, KdError> {
    query_youdao_impl(client, config, query).await
}

// Internal implementation
async fn query_youdao_impl(
    client: &Client,
    config: &Config,
    query: &str,
) -> Result<QueryResult, KdError> {
    let api_id = config.youdao.api_id.as_deref().unwrap_or("");
    let api_key = config.youdao.api_key.as_deref().unwrap_or("");

    if api_id.is_empty() {
        return Err(KdError::Config("Youdao API ID not configured".to_string()));
    }

    if api_key.is_empty() {
        return Err(KdError::Config("Youdao API Key not configured".to_string()));
    }

    let salt = Uuid::new_v4().to_string();
    let curtime = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs()
        .to_string();

    // Sign generation for Youdao API v3
    // sign = sha256(appKey + input(q) + salt + curtime + appSecret)
    let input = if query.len() <= 20 {
        query.to_string()
    } else {
        format!(
            "{}{}{}",
            &query[0..10],
            query.len(),
            &query[query.len() - 10..]
        )
    };

    let raw_sign = format!("{}{}{}{}{}", api_id, input, salt, curtime, api_key);
    let mut hasher = Sha256::new();
    hasher.update(raw_sign);
    let sign = hex::encode(hasher.finalize());

    let params = [
        ("q", query),
        ("from", "auto"),
        ("to", "auto"),
        ("appKey", api_id),
        ("salt", &salt),
        ("sign", &sign),
        ("signType", "v3"),
        ("curtime", &curtime),
    ];

    let response = client
        .get("https://openapi.youdao.com/api")
        .query(&params)
        .send()
        .await?
        .json::<YoudaoResponse>()
        .await?;

    if response.error_code != "0" {
        let error_msg = match response.error_code.as_str() {
            "101" => "Missing required parameter",
            "102" => "Unsupported language type",
            "103" => "Text too long",
            "104" => "Unsupported API type",
            "105" => "Unsupported signature type",
            "106" => "Unsupported response type",
            "107" => "Unsupported transmission encryption type",
            "108" => "Invalid appKey or signature error (check api_key)",
            "109" => "Invalid batchLog format",
            "110" => "No related service",
            "111" => "Developer account is abnormal",
            "201" => "Decryption failed, check api_key",
            "202" => "Missing signature",
            "203" => "Signature verification failed",
            "301" => "Dictionary query failed",
            "302" => "Translation query failed",
            "303" => "Server-side exception",
            "401" => "Account balance insufficient",
            "411" => "Access frequency limited",
            _ => "Unknown error",
        };
        return Err(KdError::Api(format!(
            "Youdao API Error {}: {}",
            response.error_code, error_msg
        )));
    }

    let mut result = QueryResult::new(query.to_string(), false);
    result.source = crate::domain::model::QuerySource::Online(OnlineSource::Youdao);

    if let Some(trans) = response.translations {
        result.translations = trans;
    }

    if let Some(basic) = response.basic {
        result.pronunciation = basic.pronunciation;
        if let Some(explains) = basic.explains {
            result.translations.extend(explains);
        }
    }

    if let Some(web) = response.web_translations {
        result.examples = web
            .into_iter()
            .map(|w| (w.key, w.value.join("; ")))
            .collect();
    }

    if !result.translations.is_empty()
        || result.pronunciation.is_some()
        || !result.examples.is_empty()
    {
        result.found = true;
    }

    Ok(result)
}
