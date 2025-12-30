#[derive(Debug, Clone)]
pub struct Indexed<T> {
    pub id: String,
    pub data: T,
}

impl<T> Indexed<T> {
    pub fn new(id: String, data: T) -> Self {
        Self { id, data }
    }
}

#[derive(Debug, Clone)]
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
        Self {
            next: Some(next),
        }
    }
    pub fn new(next: Option<String>) -> Self {
        Self { next }
    }
}


#[derive(Debug, Clone)]
pub struct CursorQuery {
    pub cursor: Cursor,
    pub limit: usize,
}

impl CursorQuery {
    pub fn first_page(limit: usize) -> Self {
        Self {
            cursor: Cursor { next: None },
            limit,
        }
    }
    pub fn next_page(mut self, next_cursor: Option<Cursor>) -> Option<Self> {
        let next_cursor = next_cursor?;
        self.cursor.next = next_cursor.next;
        Some(self)
    }
}

pub struct PagedResult<T> {
    pub items: Vec<Indexed<T>>,
    pub next_cursor: Option<Cursor>,
}
