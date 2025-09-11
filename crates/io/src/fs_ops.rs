use std::path::PathBuf;
use tokio::fs;
use tokio::sync::mpsc;
use uuid::Uuid;

#[derive(Debug)]
pub enum ProgressEvent {
    Update(f32),
    Completed,
    Error(String),
}

pub async fn copy_file_task(
    task_id: Uuid,
    src: PathBuf,
    dest: PathBuf,
    progress_tx: mpsc::Sender<(Uuid, ProgressEvent)>,
) {
    let result = fs::copy(&src, &dest).await;
    match result {
        Ok(_) => {
            let _ = progress_tx.send((task_id, ProgressEvent::Completed)).await;
        }
        Err(e) => {
            let _ = progress_tx.send((task_id, ProgressEvent::Error(e.to_string()))).await;
        }
    }
}

pub async fn delete_item_task(
    task_id: Uuid,
    path: PathBuf,
    progress_tx: mpsc::Sender<(Uuid, ProgressEvent)>,
) {
    let result = if path.is_dir() {
        fs::remove_dir_all(&path).await
    } else {
        fs::remove_file(&path).await
    };

    match result {
        Ok(_) => {
            let _ = progress_tx.send((task_id, ProgressEvent::Completed)).await;
        }
        Err(e) => {
            let _ = progress_tx.send((task_id, ProgressEvent::Error(e.to_string()))).await;
        }
    }
}

pub async fn move_item_task(
    task_id: Uuid,
    src: PathBuf,
    dest: PathBuf,
    progress_tx: mpsc::Sender<(Uuid, ProgressEvent)>,
) {
    let result = fs::rename(&src, &dest).await;
    match result {
        Ok(_) => {
            let _ = progress_tx.send((task_id, ProgressEvent::Completed)).await;
        }
        Err(e) => {
            let _ = progress_tx.send((task_id, ProgressEvent::Error(e.to_string()))).await;
        }
    }
}

use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};

const PREVIEW_LINE_COUNT: usize = 100;

pub async fn load_text_preview(path: PathBuf) -> Result<String, String> {
    let file = File::open(path).await.map_err(|e| e.to_string())?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    let mut content = String::new();
    for _ in 0..PREVIEW_LINE_COUNT {
        match lines.next_line().await {
            Ok(Some(line)) => {
                content.push_str(&line);
                content.push('\n');
            }
            Ok(None) => break, // End of file
            Err(e) => return Err(e.to_string()),
        }
    }

    Ok(content)
}
