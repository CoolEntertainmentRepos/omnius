use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};

use super::types::TorrentError;

/// State shared with the stream server
pub struct StreamServerState {
    /// Reference to get torrent handles for streaming
    pub manager: RwLock<Option<Arc<super::manager::TorrentManager>>>,
    /// Stored subtitle VTT content, keyed by ID
    pub subtitles: RwLock<HashMap<String, String>>,
}

/// HTTP server for streaming torrent video files
pub struct StreamServer {
    port: u16,
    state: Arc<StreamServerState>,
}

impl StreamServer {
    /// Create a new stream server on the specified port
    pub fn new(port: u16) -> Self {
        Self {
            port,
            state: Arc::new(StreamServerState {
                manager: RwLock::new(None),
                subtitles: RwLock::new(HashMap::new()),
            }),
        }
    }

    /// Start the stream server
    pub async fn start(
        &self,
        manager: Arc<super::manager::TorrentManager>,
    ) -> Result<(), TorrentError> {
        // Store the manager reference
        {
            let mut mgr = self.state.manager.write().await;
            *mgr = Some(manager);
        }

        let state = self.state.clone();

        // Build the router
        let app = Router::new()
            .route("/stream/{info_hash}/{file_index}", get(stream_handler))
            .route("/subtitle/{id}", get(subtitle_handler))
            .route("/health", get(health_handler))
            .layer(
                CorsLayer::new()
                    .allow_origin(Any)
                    .allow_methods(Any)
                    .allow_headers(Any),
            )
            .with_state(state);

        let addr = SocketAddr::from(([127, 0, 0, 1], self.port));

        // Spawn the server in the background
        tokio::spawn(async move {
            let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
            axum::serve(listener, app).await.unwrap();
        });

        Ok(())
    }

    /// Get the server port
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Store a VTT subtitle and return its ID
    pub async fn store_subtitle(&self, id: String, vtt_content: String) {
        let mut subs = self.state.subtitles.write().await;
        subs.insert(id, vtt_content);
    }
}

/// Health check endpoint
async fn health_handler() -> impl IntoResponse {
    "OK"
}

/// Serve a stored VTT subtitle by ID
async fn subtitle_handler(
    State(state): State<Arc<StreamServerState>>,
    Path(id): Path<String>,
) -> Result<Response, StatusCode> {
    let subs = state.subtitles.read().await;
    let vtt = subs.get(&id).ok_or(StatusCode::NOT_FOUND)?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/vtt; charset=utf-8")
        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .body(Body::from(vtt.clone()))
        .unwrap())
}

/// Stream handler that supports Range requests for video seeking
async fn stream_handler(
    State(state): State<Arc<StreamServerState>>,
    Path((info_hash, file_index)): Path<(String, usize)>,
    headers: HeaderMap,
) -> Result<Response, StreamError> {
    // Get the manager
    let manager_guard = state.manager.read().await;
    let manager = manager_guard
        .as_ref()
        .ok_or(StreamError::ServerNotReady)?;

    // Get the torrent handle
    let handle = manager
        .get_torrent_handle(&info_hash)
        .await
        .map_err(|_| StreamError::TorrentNotFound(info_hash.clone()))?;

    // Get file info to determine total size and content type
    let file_info = handle
        .with_metadata(|metadata| {
            metadata.file_infos.get(file_index).map(|file_info| {
                (
                    file_info.len,
                    file_info.relative_filename.to_string_lossy().to_string(),
                )
            })
        })
        .map_err(|e| StreamError::StreamError(e.to_string()))?;

    let (file_size, file_name) = file_info.ok_or(StreamError::FileNotFound(file_index))?;

    // Determine content type based on file extension
    let content_type = get_content_type(&file_name);

    // Parse Range header
    let range = parse_range_header(&headers, file_size);

    // Create the file stream from librqbit
    // The stream() method returns Result<FileStream, Error> directly (not a future)
    let file_stream = handle
        .stream(file_index)
        .map_err(|e| StreamError::StreamError(e.to_string()))?;

    match range {
        Some((start, end)) => {
            // Partial content response (206)
            let length = end - start + 1;

            // Create a reader that starts at the correct position
            let mut reader = file_stream;
            reader
                .seek(std::io::SeekFrom::Start(start))
                .await
                .map_err(|e| StreamError::StreamError(e.to_string()))?;

            // Create a limited reader for the requested range
            let limited_reader = reader.take(length);

            // Convert to body stream
            let stream = tokio_util::io::ReaderStream::new(limited_reader);
            let body = Body::from_stream(stream);

            Ok(Response::builder()
                .status(StatusCode::PARTIAL_CONTENT)
                .header(header::CONTENT_TYPE, content_type)
                .header(header::CONTENT_LENGTH, length)
                .header(
                    header::CONTENT_RANGE,
                    format!("bytes {}-{}/{}", start, end, file_size),
                )
                .header(header::ACCEPT_RANGES, "bytes")
                .body(body)
                .unwrap())
        }
        None => {
            // Full content response (200)
            let stream = tokio_util::io::ReaderStream::new(file_stream);
            let body = Body::from_stream(stream);

            Ok(Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, content_type)
                .header(header::CONTENT_LENGTH, file_size)
                .header(header::ACCEPT_RANGES, "bytes")
                .body(body)
                .unwrap())
        }
    }
}

/// Parse the Range header and return the byte range
fn parse_range_header(headers: &HeaderMap, file_size: u64) -> Option<(u64, u64)> {
    let range_header = headers.get(header::RANGE)?;
    let range_str = range_header.to_str().ok()?;

    // Parse "bytes=START-END" or "bytes=START-"
    if !range_str.starts_with("bytes=") {
        return None;
    }

    let range_part = &range_str[6..];
    let parts: Vec<&str> = range_part.split('-').collect();

    if parts.len() != 2 {
        return None;
    }

    let start: u64 = parts[0].parse().ok()?;
    let end: u64 = if parts[1].is_empty() {
        // "bytes=START-" means from START to end of file
        file_size - 1
    } else {
        parts[1].parse().ok()?
    };

    // Validate range
    if start > end || end >= file_size {
        return None;
    }

    Some((start, end))
}

/// Determine content type based on file extension
fn get_content_type(filename: &str) -> &'static str {
    let lower = filename.to_lowercase();

    if lower.ends_with(".mp4") || lower.ends_with(".m4v") {
        "video/mp4"
    } else if lower.ends_with(".mkv") {
        "video/x-matroska"
    } else if lower.ends_with(".webm") {
        "video/webm"
    } else if lower.ends_with(".avi") {
        "video/x-msvideo"
    } else if lower.ends_with(".mov") {
        "video/quicktime"
    } else if lower.ends_with(".wmv") {
        "video/x-ms-wmv"
    } else if lower.ends_with(".flv") {
        "video/x-flv"
    } else if lower.ends_with(".ts") {
        "video/mp2t"
    } else if lower.ends_with(".mpg") || lower.ends_with(".mpeg") {
        "video/mpeg"
    } else if lower.ends_with(".3gp") {
        "video/3gpp"
    } else {
        "application/octet-stream"
    }
}

/// Error types for stream handling
#[derive(Debug)]
enum StreamError {
    ServerNotReady,
    TorrentNotFound(String),
    FileNotFound(usize),
    StreamError(String),
}

impl IntoResponse for StreamError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            StreamError::ServerNotReady => (StatusCode::SERVICE_UNAVAILABLE, "Server not ready"),
            StreamError::TorrentNotFound(_) => (StatusCode::NOT_FOUND, "Torrent not found"),
            StreamError::FileNotFound(_) => (StatusCode::NOT_FOUND, "File not found"),
            StreamError::StreamError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Stream error"),
        };

        (status, message).into_response()
    }
}
