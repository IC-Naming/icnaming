

pub fn normalize_name(name: &str) -> String {
    name.trim().to_ascii_lowercase()
}

#[derive(Eq, PartialEq, Debug)]
pub struct NameParseResult {
    labels: Vec<String>,
}

impl NameParseResult {
    pub fn parse(name: &str) -> Self {
        let mut labels = Vec::new();
        // trim
        let name = name.trim();
        // split
        let mut parts = name.split('.');
        while let Some(part) = parts.next() {
            labels.push(part.trim().to_string());
        }
        NameParseResult { labels }
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
}

pub fn parse_name(name: &str) -> Result<NameParseResult, String> {
    let result = NameParseResult::parse(name);
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
