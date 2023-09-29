use axum::{
    extract::FromRef,
    routing::{post, put},
    Router,
};
use sea_orm::DatabaseConnection;

use self::route_handler::{
    create_handler, delete_handler, find_all_handler, find_by_id_handler, update_handler,
};

mod route_handler;

#[derive(FromRef, Clone)]
pub struct AppState {
    pub database: DatabaseConnection,
}

pub async fn create_routes(database: DatabaseConnection) -> Router {
    let app_state = AppState { database };
    Router::new()
        .route("/api/notes", post(create_handler).get(find_all_handler))
        .route(
            "/api/notes/:id",
            put(update_handler)
                .get(find_by_id_handler)
                .delete(delete_handler),
        )
        .with_state(app_state)
}
