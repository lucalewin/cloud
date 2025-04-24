use std::{fs, io::Write, path::PathBuf};

use axum::{
    Json,
    extract::{Multipart, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::Utc;
use sanitize_filename::sanitize;
use uuid::Uuid;

use crate::state::App;

#[derive(serde::Serialize)]
pub struct UploadResponse {
    id: String,
    original_filename: String,
    folder_path: String,
    size: i64,
}

pub async fn handler(State(state): State<App>, mut multipart: Multipart) -> impl IntoResponse {
    tracing::info!("Uploading file...");

    // Hardcoded user ID & folder path for now
    let user_id = Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap();

    let mut file_count: u32 = 0;
    while let Some(field) = multipart.next_field().await.unwrap() {
        tracing::info!("Processing field: {:?}", field.name());
        // there might be files in the multipart form
        if let Some(filename) = field.file_name().map(|s| s.to_string()) {
            tracing::info!("File name: {:?}", filename);
            let original_filename = sanitize(&filename);
            let file_id = Uuid::new_v4();

            // Check for duplicate file name in same folder
            let existing = sqlx::query_scalar!(
                r#"
                SELECT id FROM files
                WHERE user_id = $1 AND filename = $2 AND folder_id IS NULL
                "#,
                user_id,
                original_filename
            )
            .fetch_optional(&state.db)
            .await
            .unwrap();

            if existing.is_some() {
                tracing::info!("File already exists with that name");
                return (
                    StatusCode::CONFLICT,
                    Json(r#"{"status":"failed","message":"File already exists with that name"}"#),
                );
            }

            tracing::info!("File ID: {:?}", file_id);

            // Prepare upload path
            let mut upload_path = PathBuf::from(&state.upload_dir).join(user_id.to_string());
            fs::create_dir_all(&upload_path).unwrap();

            // Save file to disk
            let data = field.bytes().await.unwrap();
            let size = data.len() as i64;

            upload_path.push(file_id.to_string());
            let mut file = fs::File::create(&upload_path).unwrap();
            file.write_all(&data).unwrap();
            file.flush().unwrap();

            // Save file info to database (we do not provide a folder, so it is null, ie the root folder)
            sqlx::query!(
                r#"
                INSERT INTO files (id, user_id, filename, folder_id, size, last_modified)
                VALUES ($1, $2, $3, $4, $5, $6)
                "#,
                file_id,
                user_id,
                original_filename,
                Uuid::nil(), // root folder
                size,
                Utc::now().naive_utc()
            )
            .execute(&state.db)
            .await
            .unwrap();

            file_count += 1;
        }
    }

    if file_count > 0 {
        (StatusCode::OK, Json(r#"{"status":"success"}"#))
    } else {
        (StatusCode::BAD_REQUEST, Json(r#"{"status":"failed"}"#))
    }
}
