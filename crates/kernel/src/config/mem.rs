use std::collections::HashMap;

use switchboard_model::{Bind, ConfigService, Cursor, Indexed, NamedService};

pub type BindId = String;
fn next_bind_id() -> String {
    static ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let id = ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    format!("bind-{}", id)
}
#[derive(Clone, Debug, Default)]
pub struct Mem {
    pub named_services: HashMap<String, NamedService>,
    pub binds: HashMap<BindId, Bind>,
    pub enabled: Vec<BindId>,
}

impl Mem {
    pub fn new() -> Self {
        Self {
            named_services: HashMap::new(),
            binds: HashMap::new(),
            enabled: Vec::new(),
        }
    }

    pub fn add_named_service(&mut self, service: NamedService) {
        self.named_services.insert(service.name.clone(), service);
    }

    pub fn add_bind(&mut self, bind: Bind) -> BindId {
        let id = next_bind_id();
        self.binds.insert(id.clone(), bind);
        self.enabled.push(id.clone());
        id
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("no such index {0}")]
    NotSuchIndex(String),
}

impl ConfigService for Mem {
    type Error = Error;
    async fn get_enabled(
        &self,
        query: switchboard_model::CursorQuery,
    ) -> Result<switchboard_model::PagedResult<switchboard_model::Bind>, Self::Error> {
        let binds = if let Some(cursor) = query.cursor.next {
            // from cursor, take <limit>
            let cursor_index = self
                .enabled
                .binary_search(&cursor)
                .map_err(|_| Error::NotSuchIndex(cursor))?;
            self.enabled
                .iter()
                .skip(cursor_index)
                .take(query.limit)
                .filter_map(|id| {
                    let data = self.binds.get(id)?;
                    Some(Indexed {
                        id: id.clone(),
                        data: data.clone(),
                    })
                })
                .collect::<Vec<_>>()
        } else {
            // from start, take <limit>
            self.enabled
                .iter()
                .take(query.limit)
                .filter_map(|id| {
                    let data = self.binds.get(id)?;
                    Some(Indexed {
                        id: id.clone(),
                        data: data.clone(),
                    })
                })
                .collect::<Vec<_>>()
        };
        let next_cursor = if binds.len() < query.limit || binds.is_empty() {
            None
        } else {
            Some(Cursor {
                next: binds.last().map(|item| item.id.clone()),
            })
        };

        Ok(switchboard_model::PagedResult {
            items: binds,
            next_cursor,
        })
    }

    async fn get_named_service(&self, name: String) -> Result<Option<NamedService>, Self::Error> {
        Ok(self.named_services.get(&name).cloned())
    }
}
