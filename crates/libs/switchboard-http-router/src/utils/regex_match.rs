use std::sync::Arc;

use bytes::Bytes;

pub const REGEX_CAPTURE_GROUPS_LIMIT: usize = 16;

/// # Panics
/// Panics if index is out of bounds (>= [`REGEX_CAPTURE_GROUPS_LIMIT`])
pub const fn capture_key(index: usize) -> &'static str {
    match index {
        0 => "$0",
        1 => "$1",
        2 => "$2",
        3 => "$3",
        4 => "$4",
        5 => "$5",
        6 => "$6",
        7 => "$7",
        8 => "$8",
        9 => "$9",
        10 => "$10",
        11 => "$11",
        12 => "$12",
        13 => "$13",
        14 => "$14",
        15 => "$15",
        16 => "$16",
        _ => panic!("capture index out of bounds"),
    }
}

#[derive(Debug, Clone)]
pub struct RegexMatched {
    pub regex: Arc<str>,
    pub matched: Vec<Option<Arc<str>>>,
}

impl RegexMatched {
    pub fn from_captures(regex: &regex::Regex, captures: &regex::Captures) -> Self {
        let matched = captures
            .iter()
            .map(|m| m.map(|m| Arc::from(m.as_str())))
            .collect();
        RegexMatched {
            regex: Arc::from(regex.as_str()),
            matched,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BytesRegexMatched {
    pub regex: Arc<str>,
    pub matched: Vec<Option<Bytes>>,
}

impl BytesRegexMatched {
    pub fn from_captures(regex: &regex::bytes::Regex, captures: regex::bytes::Captures) -> Self {
        let matched = captures
            .iter()
            .map(|m| m.map(|m| Bytes::copy_from_slice(m.as_bytes())))
            .collect();
        BytesRegexMatched {
            regex: Arc::from(regex.as_str()),
            matched,
        }
    }
}
