use crate::constants::MAX_LENGTH_OF_NAME_QUOTA_TYPE;
use candid::{CandidType, Deserialize};
use std::cmp::min;
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq, Hash, CandidType, Deserialize)]
#[serde(transparent)]
pub struct NormalizedName(pub String);

impl Display for NormalizedName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, CandidType, Deserialize)]
#[serde(transparent)]
pub struct FirstLevelName(pub NameParseResult);

impl Display for FirstLevelName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.name)
    }
}

impl From<&str> for FirstLevelName {
    fn from(name: &str) -> Self {
        FirstLevelName(NameParseResult::parse(&NormalizedName(name.to_string())))
    }
}

pub fn normalize_name(name: &str) -> NormalizedName {
    NormalizedName(name.trim().to_ascii_lowercase())
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, CandidType, Deserialize)]
pub struct NameParseResult {
    labels: Vec<String>,
    name: String,
}

impl NameParseResult {
    pub fn parse(name: &NormalizedName) -> Self {
        let mut labels = Vec::new();
        // trim
        let name = name.0.trim();
        // split
        let mut parts = name.split('.');
        while let Some(part) = parts.next() {
            labels.push(part.to_string());
        }
        NameParseResult {
            labels,
            name: name.to_string(),
        }
    }

    pub fn is_top_level(&self) -> bool {
        self.labels.len() == 1
    }

    pub fn get_top_level(&self) -> Option<&String> {
        Some(&self.labels[self.labels.len() - 1])
    }
    pub fn get_current_level(&self) -> Option<&String> {
        if self.labels.len() > 0 {
            Some(&self.labels[0])
        } else {
            None
        }
    }
    pub fn get_level_count(&self) -> usize {
        self.labels.len()
    }
    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_name_len(&self) -> u8 {
        let name_length = self.labels[0].chars().count() as u8;
        name_length
    }

    pub fn get_quota_type_len(&self) -> u8 {
        min(self.get_name_len(), MAX_LENGTH_OF_NAME_QUOTA_TYPE)
    }
}

pub fn parse_name(name: &str) -> Result<NameParseResult, String> {
    let name = normalize_name(name);
    let result = NameParseResult::parse(&name);
    for label in result.labels.iter() {
        if label.len() == 0 {
            return Err("Empty label".to_string());
        }
        if !label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            return Err("name must be alphanumeric or -".to_string());
        }
    }

    return Ok(result);
}
