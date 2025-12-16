use std::{collections::HashMap, sync::Arc};

#[derive(Debug, Clone)]
pub struct Captures {
    pub captures: HashMap<Arc<str>, Arc<str>>,
}
