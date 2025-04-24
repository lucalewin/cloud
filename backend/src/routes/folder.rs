use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::state::App;

#[derive(Debug, Deserialize)]
pub struct CreateFolderRequest {
    pub name: String,
    pub parent_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct FolderResponse {
    pub id: Uuid,
    pub name: String,
    pub parent_id: Option<Uuid>,
}

pub async fn create_folder(
    State(state): State<App>,
    Json(input): Json<CreateFolderRequest>,
) -> impl IntoResponse {
    let user_id = Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(); // placeholder

    // Check for duplicate folder name in same parent
    let existing = sqlx::query_scalar!(
        r#"
        SELECT id FROM folders
        WHERE user_id = $1 AND parent_id IS NOT DISTINCT FROM $2 AND name = $3
        "#,
        user_id,
        input.parent_id,
        input.name
    )
    .fetch_optional(&state.db)
    .await
    .unwrap();

    if existing.is_some() {
        return (
            StatusCode::CONFLICT,
            "Folder already exists with that name".into_response(),
        );
    }

    let id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO folders (id, user_id, name, parent_id)
        VALUES ($1, $2, $3, $4)
        "#,
        id,
        user_id,
        input.name,
        input.parent_id
    )
    .execute(&state.db)
    .await
    .unwrap();

    (
        StatusCode::CREATED,
        Json(FolderResponse {
            id,
            name: input.name,
            parent_id: input.parent_id,
        })
        .into_response(),
    )
}

#[derive(Debug, Deserialize)]
pub struct RenameFolderRequest {
    pub folder_id: Uuid,
    pub new_name: String,
}

#[derive(Debug, Serialize)]
pub struct RenameFolderResponse {
    pub id: Uuid,
    pub name: String,
}

pub async fn rename_folder(
    State(state): State<App>,
    Json(input): Json<RenameFolderRequest>,
) -> impl IntoResponse {
    let user_id = Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(); // placeholder

    // Check for duplicate folder name in same parent
    let existing = sqlx::query_scalar!(
        r#"
        SELECT id FROM folders
        WHERE user_id = $1 AND parent_id IS NOT DISTINCT FROM (SELECT parent_id FROM folders WHERE id = $2) AND name = $3
        "#,
        user_id,
        input.folder_id,
        input.new_name
    )
    .fetch_optional(&state.db)
    .await
    .unwrap();

    if existing.is_some() {
        return (
            StatusCode::CONFLICT,
            "Folder already exists with that name".into_response(),
        );
    }

    sqlx::query!(
        r#"
        UPDATE folders
        SET name = $1
        WHERE id = $2 AND user_id = $3
        "#,
        input.new_name,
        input.folder_id,
        user_id
    )
    .execute(&state.db)
    .await
    .unwrap();

    (
        StatusCode::OK,
        Json(RenameFolderResponse {
            id: input.folder_id,
            name: input.new_name,
        })
        .into_response(),
    )
}

#[derive(Debug, Deserialize)]
pub struct MoveFolderRequest {
    pub folder_id: Uuid,
    pub new_parent_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct MoveFolderResponse {
    pub id: Uuid,
    pub new_parent_id: Option<Uuid>,
}

pub async fn move_folder(
    State(state): State<App>,
    Json(input): Json<MoveFolderRequest>,
) -> impl IntoResponse {
    let user_id = Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(); // placeholder

    // Check for duplicate folder name in new parent
    let folder_name = sqlx::query_scalar!(
        r#"
        SELECT name FROM folders
        WHERE id = $1 AND user_id = $2
        "#,
        input.folder_id,
        user_id
    )
    .fetch_one(&state.db)
    .await
    .unwrap();

    let existing = sqlx::query_scalar!(
        r#"
        SELECT id FROM folders
        WHERE user_id = $1 AND parent_id IS NOT DISTINCT FROM $2 AND name = $3
        "#,
        user_id,
        input.new_parent_id,
        folder_name
    )
    .fetch_optional(&state.db)
    .await
    .unwrap();

    if existing.is_some() {
        return (
            StatusCode::CONFLICT,
            "Folder already exists with that name".into_response(),
        );
    }

    sqlx::query!(
        r#"
        UPDATE folders
        SET parent_id = $1
        WHERE id = $2 AND user_id = $3
        "#,
        input.new_parent_id,
        input.folder_id,
        user_id
    )
    .execute(&state.db)
    .await
    .unwrap();

    (
        StatusCode::OK,
        Json(MoveFolderResponse {
            id: input.folder_id,
            new_parent_id: Some(input.new_parent_id),
        })
        .into_response(),
    )
}

#[derive(Debug, Deserialize)]
pub struct DeleteFolderRequest {
    pub folder_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct DeleteFolderResponse {
    pub id: Uuid,
}

pub async fn delete_folder(
    State(state): State<App>,
    Json(input): Json<DeleteFolderRequest>,
) -> impl IntoResponse {
    let user_id = Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(); // placeholder

    // start transaction
    let mut transaction = state.db.begin().await.unwrap();

    // get all subfolders
    let subfolders = sqlx::query_scalar!(
        r#"
        WITH RECURSIVE subfolders AS (
            SELECT id FROM folders WHERE id = $1 AND user_id = $2
            UNION ALL
            SELECT f.id FROM folders f
            JOIN subfolders sf ON f.parent_id = sf.id
        )
        SELECT id FROM subfolders
        "#,
        input.folder_id,
        user_id
    )
    .fetch_all(&mut *transaction)
    .await
    .unwrap();

    // delete all subfolders (and the folder itself)
    for folder_id in subfolders {
        sqlx::query!(
            r#"
            DELETE FROM folders WHERE id = $1 AND user_id = $2
            "#,
            folder_id,
            user_id
        )
        .execute(&mut *transaction)
        .await
        .unwrap();

        // delete all files in the folder
        sqlx::query!(
            r#"
            DELETE FROM files WHERE folder_id = $1 AND user_id = $2
            "#,
            folder_id,
            user_id
        )
        .execute(&mut *transaction)
        .await
        .unwrap();
    }

    // commit transaction
    transaction.commit().await.unwrap();

    (
        StatusCode::OK,
        Json(DeleteFolderResponse {
            id: input.folder_id,
        })
        .into_response(),
    )
}
