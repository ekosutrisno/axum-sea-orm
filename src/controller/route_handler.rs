use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sea_orm::{
    prelude::Uuid, ActiveModelTrait, ActiveValue::NotSet, DatabaseConnection, EntityTrait, Set,
};
use serde_json::{json, Value};

use crate::{
    model::notes,
    schema::{CreateNoteSchema, FilterOptions, UpdateNoteSchema},
};
use crate::{model::notes::Entity as Notes, schema::NoteResponse};

pub async fn find_all_handler(
    State(db): State<DatabaseConnection>,
    opts: Option<Query<FilterOptions>>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    let Query(_opts) = opts.unwrap_or_default();

    let notes_result = Notes::find().all(&db).await.map_err(|_| {
        let error_response = json!({
            "status": "fail",
            "message": "Something bad happened while fetching all note items",
        });
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    })?;

    let notes_list: Vec<NoteResponse> = notes_result
        .into_iter()
        .map(|n| NoteResponse {
            id: n.id,
            title: n.title,
            content: n.content,
            category: n.category,
            published: n.published,
            created_at: n.created_at,
            updated_at: n.updated_at,
        })
        .collect();

    let json_response = match notes_list.len() {
        len if len > 0 => json!({
            "status": "success",
            "results": len,
            "data": notes_list
        }),
        _ => json!({
            "status": "success",
            "results": 0,
            "message": "No notes found",
        }),
    };

    Ok(Json(json_response))
}

pub async fn find_by_id_handler(
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    match Notes::find_by_id(id).one(&db).await {
        Ok(Some(note)) => {
            let response = NoteResponse {
                id,
                title: note.title,
                content: note.content,
                category: note.category,
                published: note.published,
                created_at: note.created_at,
                updated_at: note.updated_at,
            };

            let note_response = json!({
                "status": "success",
                "data": {
                    "note": response
                }
            });
            Ok((StatusCode::OK, Json(note_response)))
        }
        Ok(None) => {
            let error_response = json!({
                "status": "fail",
                "message": format!("Note with ID: {} not found", id)
            });
            Ok((StatusCode::NOT_FOUND, Json(error_response)))
        }
        Err(_) => {
            let error_response = json!({
                "status": "fail",
                "message": "Something went wrong while fetching the note"
            });
            Ok((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

pub async fn create_handler(
    State(db): State<DatabaseConnection>,
    Json(data): Json<CreateNoteSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    let new_note = notes::ActiveModel {
        id: NotSet,
        title: Set(data.title.clone()),
        content: Set(data.content.clone()),
        category: Set(data.category.clone()),
        ..Default::default()
    };

    match new_note.insert(&db).await {
        Ok(saved_note) => {
            let response = NoteResponse {
                id: saved_note.id,
                title: saved_note.title,
                content: saved_note.content.clone(),
                category: saved_note.category.clone(),
                published: saved_note.published,
                created_at: saved_note.created_at,
                updated_at: saved_note.updated_at,
            };
            let note_response = json!({
                "status": "success",
                "data": {
                    "note": response
                }
            });
            Ok((StatusCode::CREATED, Json(note_response)))
        }
        Err(e) => {
            if e.to_string()
                .contains("duplicate key value violates unique constraint")
            {
                let error_response = json!({
                    "status": "fail",
                    "message": "Note with that title already exists",
                });
                Err((StatusCode::CONFLICT, Json(error_response)))
            } else {
                let error_response = json!({
                    "status": "error",
                    "message": format!("{:?}", e),
                });
                Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
            }
        }
    }
}

pub async fn update_handler(
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
    Json(data): Json<UpdateNoteSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    let note_result = Notes::find_by_id(id).one(&db).await;

    match note_result {
        Ok(note) => {
            let mut note: notes::ActiveModel = note.unwrap().into();

            if let Some(title) = data.title {
                note.title = Set(title);
            }
            if let Some(content) = data.content {
                note.content = Set(content);
            }
            if let Some(category) = data.category {
                note.category = Set(Some(category));
            }
            if let Some(published) = data.published {
                note.published = Set(Some(published));
            }

            if let Ok(updated_note) = note.update(&db).await {
                let response = NoteResponse {
                    id: updated_note.id,
                    title: updated_note.title.clone(),
                    content: updated_note.content.clone(),
                    category: updated_note.category.clone(),
                    published: updated_note.published,
                    created_at: updated_note.created_at,
                    updated_at: updated_note.updated_at,
                };

                let note_response = json!({
                    "status": "success",
                    "data": {
                        "note": response
                    }
                });
                Ok((StatusCode::OK, Json(note_response)))
            } else {
                let error_response = json!({
                    "status": "fail",
                    "message": "Failed to update the note"
                });
                Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
            }
        }
        Err(_) => {
            let error_response = json!({
                "status": "error",
                "message": format!("Error while fetching the note with ID: {}", id)
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

pub async fn delete_handler(
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    match Notes::delete_by_id(id).exec(&db).await {
        Ok(rows_affected) => {
            if rows_affected.rows_affected == 0 {
                let error_response = json!({
                    "status": "fail",
                    "message": format!("Note with ID: {} not found", id),
                });
                return Err((StatusCode::NOT_FOUND, Json(error_response)));
            }

            Ok(StatusCode::NO_CONTENT)
        }
        Err(error) => {
            let error_response = json!({
                "status": "error",
                "message": format!("Failed to delete note with ID: {}", id),
                "details": error.to_string(),
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}
