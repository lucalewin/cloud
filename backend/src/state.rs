use std::sync::Arc;

pub type App = Arc<AppState>;

pub struct AppState {
    pub upload_dir: String,
    pub db: sqlx::PgPool,
}

impl AppState {
    pub fn new(upload_dir: String, db: sqlx::PgPool) -> Self {
        Self { upload_dir, db }
    }
}
