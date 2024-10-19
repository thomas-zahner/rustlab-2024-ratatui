use dashmap::DashSet;
use std::sync::Arc;

const MAX_USERS: usize = 128;

#[derive(Clone)]
#[repr(transparent)]
pub struct Users(Arc<DashSet<String>>);

impl Users {
    pub fn new() -> Self {
        Self(Arc::new(DashSet::with_capacity(MAX_USERS)))
    }

    pub fn insert(&self, name: String) -> bool {
        self.0.insert(name)
    }

    pub fn remove(&self, name: &str) -> bool {
        self.0.remove(name).is_some()
    }
}
