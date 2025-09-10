use std::sync::{Arc, Mutex};
use uuid::Uuid;
use std::path::PathBuf;
use tokio::sync::mpsc;
use io::fs_ops;

#[derive(Debug, Clone)]
pub enum TaskKind {
    Copy { src: PathBuf, dest: PathBuf },
    Move { src: PathBuf, dest: PathBuf },
    Delete { path: PathBuf },
}

#[derive(Debug, Clone, PartialEq)]
pub enum TaskStatus {
    Pending,
    InProgress(f32), // Progress from 0.0 to 1.0
    Completed,
    Failed(String),
}

#[derive(Debug, Clone)]
pub struct Task {
    pub id: Uuid,
    pub kind: TaskKind,
    pub status: TaskStatus,
    pub description: String,
}

impl Task {
    pub fn new(kind: TaskKind, description: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            kind,
            status: TaskStatus::Pending,
            description,
        }
    }
}

use std::fmt;

pub struct TaskManager {
    tasks: Arc<Mutex<Vec<Task>>>,
    progress_rx: Mutex<mpsc::Receiver<(Uuid, fs_ops::ProgressEvent)>>,
    progress_tx: mpsc::Sender<(Uuid, fs_ops::ProgressEvent)>,
    pub completed_and_needs_refresh: Arc<Mutex<Vec<PathBuf>>>,
}

impl fmt::Debug for TaskManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TaskManager")
         .field("tasks", &self.tasks.lock().unwrap())
         .finish()
    }
}

impl TaskManager {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(100);
        Self {
            tasks: Arc::new(Mutex::new(Vec::new())),
            progress_rx: Mutex::new(rx),
            progress_tx: tx,
            completed_and_needs_refresh: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn add_task(&self, kind: TaskKind, description: String) {
        let task = Task::new(kind, description);
        self.tasks.lock().unwrap().push(task);
    }

    pub fn get_tasks(&self) -> Vec<Task> {
        self.tasks.lock().unwrap().clone()
    }

    pub fn process_pending_tasks(&self) {
        let mut tasks = self.tasks.lock().unwrap();
        for task in tasks.iter_mut() {
            if task.status == TaskStatus::Pending {
                task.status = TaskStatus::InProgress(0.0);

                let task_id = task.id;
                let kind = task.kind.clone();
                let progress_tx = self.progress_tx.clone();

                tokio::spawn(async move {
                    match kind {
                        TaskKind::Copy { src, dest } => {
                            fs_ops::copy_file_task(task_id, src, dest, progress_tx).await;
                        }
                        TaskKind::Move { src, dest } => {
                            fs_ops::move_item_task(task_id, src, dest, progress_tx).await;
                        }
                        TaskKind::Delete { path } => {
                            fs_ops::delete_item_task(task_id, path, progress_tx).await;
                        }
                    }
                });
            }
        }
    }

    pub fn update_task_statuses(&self) {
        let mut rx = self.progress_rx.lock().unwrap();
        while let Ok((task_id, event)) = rx.try_recv() {
            let mut tasks = self.tasks.lock().unwrap();
            if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) {
                match event {
                    fs_ops::ProgressEvent::Completed { dest_path } => {
                        task.status = TaskStatus::Completed;
                        self.completed_and_needs_refresh.lock().unwrap().push(dest_path);
                    }
                    fs_ops::ProgressEvent::Error(e) => task.status = TaskStatus::Failed(e),
                    fs_ops::ProgressEvent::Update(p) => task.status = TaskStatus::InProgress(p),
                }
            }
        }
    }
}

impl Default for TaskManager {
    fn default() -> Self {
        Self::new()
    }
}
