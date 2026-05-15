use std::sync::Arc;

use crate::repositories::database::Database;

pub struct AppState {
    pub db: Arc<Database>,
}

impl AppState {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }
}
