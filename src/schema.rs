use chrono::{DateTime, FixedOffset};
use sea_orm::prelude::Uuid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Default)]
pub struct FilterOptions {
    pub page: Option<usize>,
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct ParamOption {
    pub id: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateNoteSchema {
    pub title: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateNoteSchema {
    pub title: Option<String>,
    pub content: Option<String>,
    pub category: Option<String>,
    pub published: Option<bool>,
}

#[derive(Serialize)]
pub struct NoteResponse {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub category: Option<String>,
    pub published: Option<bool>,
    pub created_at: Option<DateTime<FixedOffset>>,
    pub updated_at: Option<DateTime<FixedOffset>>,
}
