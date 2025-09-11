use crate::app_state::DirEntry;
use mupdf_ffi::rasterize_image;
use raster::PreviewImage;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

// The different states of a preview.
#[derive(Debug, Clone, Default)]
pub enum PreviewState {
    #[default]
    Empty,
    Loading,
    Text(String),
    Image(Arc<PreviewImage>),
    Directory(Vec<DirEntry>),
    Error(String),
}

// A unique identifier for a render task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TaskId(pub Uuid);

enum RenderStage {
    Thumbnail,
    Final,
}

// A request sent to the rendering worker pool.
struct RenderRequest {
    id: TaskId,
    path: PathBuf,
    width: u32,
    height: u32,
    stage: RenderStage,
}

// An event sent from the controller back to the UI thread.
#[derive(Debug)]
pub enum PreviewEvent {
    ThumbnailLoaded(TaskId, Arc<PreviewImage>),
    FinalImageLoaded(TaskId, Arc<PreviewImage>),
    Error(TaskId, String),
}

// The controller that manages preview rendering.
pub struct PreviewController {
    request_tx: mpsc::Sender<RenderRequest>,
}

impl PreviewController {
    pub fn new(event_tx: mpsc::Sender<PreviewEvent>) -> Self {
        let (request_tx, mut request_rx) = mpsc::channel::<RenderRequest>(20); // Increased capacity for two-stage render

        tokio::spawn(async move {
            while let Some(request) = request_rx.recv().await {
                let result = tokio::task::spawn_blocking(move || {
                    rasterize_image(&request.path, request.width, request.height)
                })
                .await;

                let event = match result {
                    Ok(Ok(image)) => {
                        let image_arc = Arc::new(image);
                        match request.stage {
                            RenderStage::Thumbnail => PreviewEvent::ThumbnailLoaded(request.id, image_arc),
                            RenderStage::Final => PreviewEvent::FinalImageLoaded(request.id, image_arc),
                        }
                    }
                    Ok(Err(e)) => PreviewEvent::Error(request.id, e.to_string()),
                    Err(e) => PreviewEvent::Error(request.id, e.to_string()),
                };

                if event_tx.send(event).await.is_err() {
                    break;
                }
            }
        });

        Self { request_tx }
    }

    pub async fn request_preview(
        &self,
        path: PathBuf,
        final_width: u32,
        final_height: u32,
        progressive: bool,
    ) -> TaskId {
        let id = TaskId(Uuid::new_v4());

        if progressive {
            // Request 1: Thumbnail
            let thumb_width = (final_width / 4).max(1);
            let thumb_height = (final_height / 4).max(1);
            let thumb_request = RenderRequest {
                id,
                path: path.clone(),
                width: thumb_width,
                height: thumb_height,
                stage: RenderStage::Thumbnail,
            };
            self.request_tx.send(thumb_request).await.expect("Worker died");
        }

        // Request 2: Final Image (always sent)
        let final_request = RenderRequest {
            id,
            path,
            width: final_width,
            height: final_height,
            stage: RenderStage::Final,
        };
        self.request_tx.send(final_request).await.expect("Worker died");

        id
    }
}
