pub mod handlers;
pub mod tasks;

use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;

use crate::client::BiliClient;
use tasks::TaskManager;

#[derive(Clone)]
pub struct AppState {
    pub tasks: Arc<TaskManager>,
    pub client: BiliClient,
}

pub fn router(client: BiliClient) -> Router {
    let state = AppState {
        tasks: Arc::new(TaskManager::new()),
        client,
    };

    Router::new()
        .route("/", get(handlers::index))
        .route("/static/style.css", get(handlers::style))
        .route("/static/app.js", get(handlers::script))
        .route("/api/tasks", post(handlers::create_task))
        .route("/api/tasks/{id}/events", get(handlers::task_events))
        .route("/api/tasks/{id}/file", get(handlers::download_file))
        .with_state(state)
}