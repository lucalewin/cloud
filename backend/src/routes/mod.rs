use axum::Router;

use crate::state::App;

pub mod download;
pub mod files;
pub mod folder;
pub mod upload;

pub fn router() -> Router<App> {
    Router::new()
}
