use serde::{Deserialize, Serialize};

use crate::regex::SerdeRegex;

pub enum Rule {
    Path(PathRule),
    Method(String),
    Header(Vec<HeaderRule>),
    Query(Vec<QueryRule>),
}

pub enum PathRule {
    Exact(String),
    Prefix(String),
    Regex(SerdeRegex),
}

pub enum QueryRule {
    Exact {
        name: String,
        value: String,
    },
    Regex {
        name: String,
        value: SerdeRegex,
    },
}


pub enum HeaderRule {
    Exact {
        name: String,
        value: String,
    },
    Regex {
        name: String,
        value: SerdeRegex,
    },
}


pub struct HbRouter {

}