use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{
        sse::{Event, KeepAlive, Sse},
        Html, IntoResponse, Response,
    },
    Json,
};
use futures_util::{
    stream::{self, BoxStream},
    StreamExt,
};
use serde::Deserialize;
use std::convert::Infallible;
use std::time::Duration;
use tokio_stream::wrappers::BroadcastStream;

use super::AppState;
use crate::{pipeline, utils};

type EventStream = BoxStream<'static, Result<Event, Infallible>>;

pub async fn index() -> Html<&'static str> {
    Html(include_str!("../static/index.html"))
}

pub async fn style() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
        include_str!("../static/style.css"),
    )
}

pub async fn script() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "application/javascript; charset=utf-8")],
        include_str!("../static/app.js"),
    )
}

#[derive(Deserialize)]
pub struct CreateTaskReq {
    url: String,
}

pub async fn create_task(
    State(state): State<AppState>,
    Json(req): Json<CreateTaskReq>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    utils::extract_bvid(&req.url)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let task = state.tasks.create(req.url).await;
    let task_id = task.id.clone();
    let client = state.client.clone();

    tokio::spawn(async move {
        pipeline::run(task, client).await;
    });

    Ok(Json(serde_json::json!({ "task_id": task_id })))
}

pub async fn task_events(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    let task = state.tasks.get(&id).await.ok_or(StatusCode::NOT_FOUND)?;

    let last = task.last.read().await.clone();
    let is_terminal = last.is_terminal();

    let initial: EventStream = Box::pin(stream::once(async move {
        let data = serde_json::to_string(&last).unwrap();
        Ok::<_, Infallible>(Event::default().data(data))
    }));

    let stream: EventStream = if is_terminal {
        initial
    } else {
        let rx = task.tx.subscribe();
        let live = BroadcastStream::new(rx).filter_map(|res| async move {
            let ev = res.ok()?;
            let data = serde_json::to_string(&ev).ok()?;
            Some(Ok::<_, Infallible>(Event::default().data(data)))
        });
        Box::pin(initial.chain(live))
    };

    Ok(Sse::new(stream).keep_alive(
        KeepAlive::new().interval(Duration::from_secs(15)),
    ))
}

pub async fn download_file(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Response, StatusCode> {
    let task = state.tasks.get(&id).await.ok_or(StatusCode::NOT_FOUND)?;
    let path = task
        .file_path
        .read()
        .await
        .clone()
        .ok_or(StatusCode::NOT_FOUND)?;

    let bytes = tokio::fs::read(&path)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let filename = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("output.mp4")
        .to_string();

    Ok((
        [
            (header::CONTENT_TYPE, "video/mp4".to_string()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{}\"", filename),
            ),
        ],
        bytes,
    )
        .into_response())
}