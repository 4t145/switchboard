pub struct Indexed<T> {
    pub id: String,
    pub data: T,
}

pub struct Cursor {
    pub next: Option<String>,
}

pub struct CursorQuery {
    pub cursor: Cursor,
    pub limit: usize,
}

pub struct PagedResult<T> {
    pub items: Vec<Indexed<T>>,
    pub next_cursor: Option<Cursor>,
}
