use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Indexed<T> {
    pub id: String,
    pub data: T,
}

impl<T> Indexed<T> {
    pub fn new(id: String, data: T) -> Self {
        Self { id, data }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cursor {
    pub next: Option<String>,
}

impl Cursor {
    pub fn is_empty(&self) -> bool {
        self.next.is_none()
    }
    pub fn empty() -> Self {
        Self { next: None }
    }
    pub fn from_next(next: String) -> Self {
        Self { next: Some(next) }
    }
    pub fn new(next: Option<String>) -> Self {
        Self { next }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageQuery {
    pub cursor: Cursor,
    pub limit: usize,
}

impl PageQuery {
    pub fn with_limit(limit: usize) -> Self {
        Self {
            cursor: Cursor::empty(),
            limit,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlattenPageQueryWithFilter<F> {
    #[serde(default)]
    pub next: Option<String>,
    pub limit: usize,
    #[serde(flatten)]
    pub filter: F,
}

impl<F> FlattenPageQueryWithFilter<F> {
    pub fn into_parts(self) -> (PageQuery, F) {
        (
            PageQuery {
                cursor: Cursor { next: self.next },
                limit: self.limit,
            },
            self.filter,
        )
    }
}

impl<F> Into<(PageQuery, F)> for FlattenPageQueryWithFilter<F> {
    fn into(self) -> (PageQuery, F) {
        (
            PageQuery {
                cursor: Cursor { next: self.next },
                limit: self.limit,
            },
            self.filter,
        )
    }
}

impl PageQuery {
    pub fn with_cursor(mut self, cursor: Cursor) -> Self {
        self.cursor = cursor;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PagedList<T> {
    pub items: Vec<Indexed<T>>,
    pub next_cursor: Option<Cursor>,
}
