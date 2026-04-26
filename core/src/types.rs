use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Item {
    pub id: String,
    pub title: String,
    pub bin: String,
    pub source: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Agg {
    #[serde(default)] pub red: u64,
    #[serde(default)] pub blue: u64,
    #[serde(default)] pub abstain: u64,
    #[serde(default)] pub total: u64,
}

pub fn label(c: i8) -> &'static str {
    match c { -1 => "red", 0 => "abstain", 1 => "blue", _ => "?" }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn labels() { assert_eq!(label(-1), "red"); assert_eq!(label(0), "abstain"); assert_eq!(label(1), "blue"); }
    #[test] fn gf3_balanced() { let s: i64 = [-1_i64,0,1].iter().sum(); assert_eq!(((s%3)+3)%3, 0); }
}
