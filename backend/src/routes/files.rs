use axum::{extract::{Path, State}, response::IntoResponse, Json};
use uuid::Uuid;

use crate::state::App;

use super::folder;

#[derive(Debug, serde::Deserialize)]
pub struct FileRequest {
    pub folder_id: Option<Uuid>,
}

pub async fn get_handler(State(state): State<App>, Json(payload): Json<FileRequest>) -> impl IntoResponse {
    // let folder_id = if folder_id.is_empty() {
    //     Uuid::nil()
    // } else {
    //     Uuid::parse_str(&folder_id).unwrap()
    // };

    // if payload.folder_id.is_some_and(|id| id.is_nil()) {
    //     payload.folder_id = None;
    // }

    dbg!(&payload);

    tracing::info!("Fetching files in folder: {}", payload.folder_id.unwrap_or_default().to_string());
    tracing::info!("Fetching files in folder: {:?}", payload.folder_id);
    
    let user_id = Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(); // placeholder
    // get all files in the folder
    let files = sqlx::query!(
        r#"
        SELECT id, filename, size, last_modified FROM files WHERE folder_id = $1 AND user_id = $2
        "#,
        payload.folder_id.unwrap_or_default(),
        user_id
    )
    .fetch_all(&state.db)
    .await
    .unwrap();

    // get all folders in the folder
    let folders = sqlx::query!(
        r#"
        SELECT id, name, parent_id FROM folders WHERE parent_id = $1 AND user_id = $2
        "#,
        payload.folder_id,
        user_id
    )
    .fetch_all(&state.db)
    .await
    .unwrap();

    // return files and folders as JSON
    let files: Vec<_> = files
        .into_iter()
        .map(|f| (f.id, f.filename, f.size, f.last_modified))
        .collect();
    dbg!(&files);
    let folders: Vec<_> = folders
        .into_iter()
        .map(|f| (f.id, f.name, f.parent_id))
        .collect();
    dbg!(&folders);

    let response = serde_json::json!({
        "files": files,
        "folders": folders,
    });
    (axum::http::StatusCode::OK, axum::Json(response))
}

pub async fn delete_handler() {}
