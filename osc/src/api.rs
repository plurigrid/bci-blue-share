use color_eyre::eyre::{eyre, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Item {
    pub id: String,
    pub title: String,
    pub bin: String,
    pub source: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Agg {
    #[serde(default)]
    pub red: u64,
    #[serde(default)]
    pub blue: u64,
    #[serde(default)]
    pub abstain: u64,
    #[serde(default)]
    pub total: u64,
}

fn agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(10))
        .user_agent("osc/0.1 (+https://bci.blue)")
        .build()
}

pub fn fetch_catalog(endpoint: &str) -> Result<Vec<Item>> {
    let url = format!("{}/api/catalog", endpoint.trim_end_matches('/'));
    let s = agent().get(&url).call()?.into_string()?;
    let v: serde_json::Value = serde_json::from_str(&s)?;
    let items = v.get("items").ok_or_else(|| eyre!("no items"))?.clone();
    Ok(serde_json::from_value(items)?)
}

pub fn fetch_agg(endpoint: &str) -> Result<HashMap<String, Agg>> {
    let url = format!("{}/api/agg", endpoint.trim_end_matches('/'));
    let s = agent().get(&url).call()?.into_string()?;
    let v: serde_json::Value = serde_json::from_str(&s)?;
    let aggs = v.get("aggregates").cloned().unwrap_or(serde_json::json!({}));
    Ok(serde_json::from_value(aggs).unwrap_or_default())
}

pub fn cast_vote(endpoint: &str, item: &str, choice: i8) -> Result<Agg> {
    let url = format!("{}/api/vote", endpoint.trim_end_matches('/'));
    let body = serde_json::json!({"item": item, "choice": choice});
    let s = agent().post(&url).send_json(body)?.into_string()?;
    let v: serde_json::Value = serde_json::from_str(&s)?;
    let agg = v.get("agg").cloned().unwrap_or_default();
    Ok(serde_json::from_value(agg).unwrap_or_default())
}
