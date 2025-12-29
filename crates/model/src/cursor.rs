#[derive(Debug, Clone)]
pub struct Indexed<T> {
    pub id: String,
    pub data: T,
}

#[derive(Debug, Clone)]
pub struct Cursor {
    pub next: Option<String>,
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
