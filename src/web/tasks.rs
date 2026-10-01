use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use serde::Serialize;
use tokio::sync::{broadcast, RwLock};

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TaskEvent {
    Queued,
    Status { message: String },
    Progress {
        stage: String,
        downloaded: u64,
        total: Option<u64>,
    },
    Done { filename: String, size: u64 },
    Failed { message: String },
}

impl TaskEvent {
    pub fn is_terminal(&self) -> bool {
        matches!(self, TaskEvent::Done { .. } | TaskEvent::Failed { .. })
    }
}

pub struct Task {
    pub id: String,
    pub url: String,
    pub tx: broadcast::Sender<TaskEvent>,
    pub last: RwLock<TaskEvent>,
    pub file_path: RwLock<Option<PathBuf>>,
}

impl Task {
    fn new(id: String, url: String) -> Self {
        let (tx, _) = broadcast::channel(64);
        Self {
            id,
            url,
            tx,
            last: RwLock::new(TaskEvent::Queued),
            file_path: RwLock::new(None),
        }
    }

    pub async fn emit(&self, ev: TaskEvent) {
        *self.last.write().await = ev.clone();
        let _ = self.tx.send(ev);
    }

    pub async fn status(&self, msg: &str) {
        self.emit(TaskEvent::Status {
            message: msg.to_string(),
        })
        .await;
    }

    pub async fn progress(&self, stage: &str, downloaded: u64, total: Option<u64>) {
        self.emit(TaskEvent::Progress {
            stage: stage.to_string(),
            downloaded,
            total,
        })
        .await;
    }
}

#[derive(Default)]
pub struct TaskManager {
    tasks: RwLock<HashMap<String, Arc<Task>>>,
}

impl TaskManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn create(&self, url: String) -> Arc<Task> {
        let id = uuid::Uuid::new_v4().to_string();
        let task = Arc::new(Task::new(id.clone(), url));
        self.tasks.write().await.insert(id, Arc::clone(&task));
        task
    }

    pub async fn get(&self, id: &str) -> Option<Arc<Task>> {
        self.tasks.read().await.get(id).cloned()
    }
}