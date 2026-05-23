use std::io::{self, Read, Seek, SeekFrom, Write};
use std::net::SocketAddr;
use std::path::{Path as FsPath, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

use axum::{
    body::{Body, Bytes},
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        DefaultBodyLimit, Multipart, Path, Query, State,
    },
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Json, Response},
    routing::{delete, get, post},
    Router,
};
use base64::Engine;
use qrcode::QrCode;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio::sync::watch;
use tokio_stream::wrappers::ReceiverStream;
use tokio_util::io::ReaderStream;

use crate::core::error::AppError;
use crate::core::types::WifiTransferStatus;
use crate::services::transfer::TransferService;

const DEFAULT_PORT: u16 = 37210;
const WIFI_EVENT_CHANNEL_SIZE: usize = 256;
const PARTIAL_UPLOAD_DIR_NAME: &str = ".dd-upload-partials";

static WIFI_EVENT_TX: OnceLock<Mutex<Option<broadcast::Sender<WifiFileEvent>>>> = OnceLock::new();

#[derive(Clone)]
struct AppState {
    token: String,
    upload_dir: PathBuf,
    app_handle: tauri::AppHandle,
    locale: String,
    event_tx: broadcast::Sender<WifiFileEvent>,
    history: Arc<Mutex<Vec<TransferHistoryEntry>>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct VerifyQuery {
    token: Option<String>,
    path: Option<String>,
    client_id: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ChunkUploadQuery {
    token: Option<String>,
    client_id: Option<String>,
    upload_id: String,
    path: String,
    file_size: u64,
    offset: u64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct UploadStatusQuery {
    token: Option<String>,
    upload_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BatchDownloadQuery {
    token: Option<String>,
    paths: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FileInfo {
    name: String,
    path: String,
    size: u64,
    is_directory: bool,
    modified: Option<i64>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct TransferHistoryEntry {
    id: String,
    #[serde(rename = "type")]
    entry_type: String,
    name: String,
    path: String,
    size: u64,
    status: String,
    client_id: Option<String>,
    checksum: Option<String>,
    message: Option<String>,
    timestamp: i64,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WifiFileEvent {
    id: String,
    #[serde(rename = "type")]
    event_type: String,
    path: Option<String>,
    actor_client_id: Option<String>,
    timestamp: i64,
}

pub fn broadcast_wifi_file_event(
    event_type: &str,
    path: Option<String>,
    actor_client_id: Option<String>,
) {
    let event = WifiFileEvent {
        id: nanoid::nanoid!(10),
        event_type: event_type.to_string(),
        path,
        actor_client_id,
        timestamp: chrono::Utc::now().timestamp_millis(),
    };

    if let Ok(guard) = wifi_event_bus().lock() {
        if let Some(tx) = guard.as_ref() {
            let _ = tx.send(event);
        }
    }
}

fn wifi_event_bus() -> &'static Mutex<Option<broadcast::Sender<WifiFileEvent>>> {
    WIFI_EVENT_TX.get_or_init(|| Mutex::new(None))
}

fn set_wifi_event_tx(tx: Option<broadcast::Sender<WifiFileEvent>>) {
    if let Ok(mut guard) = wifi_event_bus().lock() {
        *guard = tx;
    }
}

pub async fn start_server(
    transfer_service: &TransferService,
    port: Option<u16>,
    custom_dir: Option<String>,
    max_upload_gb: u32,
    locale: String,
) -> Result<WifiTransferStatus, AppError> {
    let current = transfer_service.get_wifi_transfer_status();
    if current.running {
        return Ok(current);
    }

    let port = port.unwrap_or(DEFAULT_PORT);
    let token = nanoid::nanoid!(8);
    let upload_dir = if let Some(custom) = custom_dir {
        PathBuf::from(custom)
    } else {
        crate::core::data_dir::devicedeck_data_dir()
            .unwrap_or_else(|_| std::env::temp_dir().join(".devicedeck"))
            .join("devicedeck-wifi-transfer")
    };
    tokio::fs::create_dir_all(&upload_dir).await?;

    let max_upload_bytes = (max_upload_gb.clamp(1, 50) as usize) * 1024 * 1024 * 1024;

    let local_ip = local_ip_address::local_ip()
        .unwrap_or_else(|_| std::net::IpAddr::V4([127, 0, 0, 1].into()));
    let url = format_access_url(local_ip, port);
    let qr_data = format!("{}?token={}", url, token);

    let qr = QrCode::new(qr_data.as_bytes())
        .map_err(|e| AppError::internal_error(&format!("QR generation failed: {e}")))?;
    let qr_png = qr
        .render::<qrcode::render::svg::Color>()
        .min_dimensions(200, 200)
        .build();

    let qr_data_url = format!(
        "data:image/svg+xml;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(qr_png.as_bytes())
    );

    let (event_tx, _) = broadcast::channel(WIFI_EVENT_CHANNEL_SIZE);
    let history = Arc::new(Mutex::new(Vec::new()));

    let state = AppState {
        token: token.clone(),
        upload_dir: upload_dir.clone(),
        app_handle: transfer_service.app_handle(),
        locale: normalize_locale(&locale).to_string(),
        event_tx: event_tx.clone(),
        history,
    };

    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    let app = Router::new()
        .route("/", get(serve_mobile_page))
        .route("/api/verify", get(verify_token))
        .route("/api/upload", post(upload_file))
        .route("/api/upload/chunk", post(upload_chunk))
        .route("/api/upload/status", get(upload_status))
        .route("/api/files", get(list_files))
        .route("/api/download/{*path}", get(download_file))
        .route("/api/download-zip", get(download_selected_zip))
        .route("/api/preview/{*path}", get(preview_file))
        .route("/api/files/{*path}", delete(delete_file))
        .route("/api/history", get(transfer_history))
        .route("/ws", get(wifi_events))
        .layer(DefaultBodyLimit::max(max_upload_bytes))
        .layer(tower_http::cors::CorsLayer::permissive())
        .with_state(state);

    let addr = SocketAddr::from(([0u8, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| AppError::internal_error(&format!("Failed to bind port {}: {}", port, e)))?;

    let status = WifiTransferStatus {
        running: true,
        url: Some(url),
        token: Some(token),
        qr_code_data_url: Some(qr_data_url),
        port,
    };

    transfer_service.update_wifi_status(status.clone());
    transfer_service.set_wifi_upload_dir(upload_dir.clone());
    set_wifi_event_tx(Some(event_tx));

    let status_for_shutdown = status.clone();
    tokio::spawn(async move {
        let shutdown_rx = shutdown_rx.clone();
        axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                let mut rx = shutdown_rx;
                let _ = rx.changed().await;
            })
            .await
            .ok();

        let stopped = WifiTransferStatus {
            running: false,
            url: None,
            token: None,
            qr_code_data_url: None,
            port: status_for_shutdown.port,
        };
        // We can't easily update status after shutdown since we don't have the service handle here
        let _ = stopped;
    });

    // Store shutdown sender
    transfer_service.set_shutdown_tx(shutdown_tx);

    Ok(status)
}

pub async fn stop_server(transfer_service: &TransferService) -> Result<(), AppError> {
    let current = transfer_service.get_wifi_transfer_status();
    if !current.running {
        return Ok(());
    }

    transfer_service.send_shutdown()?;
    transfer_service.clear_wifi_upload_dir();
    set_wifi_event_tx(None);

    transfer_service.update_wifi_status(WifiTransferStatus {
        running: false,
        url: None,
        token: None,
        qr_code_data_url: None,
        port: current.port,
    });

    Ok(())
}

async fn serve_mobile_page(State(state): State<AppState>) -> Html<String> {
    Html(render_mobile_html(&state.locale))
}

fn normalize_locale(locale: &str) -> &'static str {
    if locale.eq_ignore_ascii_case("en") {
        "en"
    } else {
        "zh-CN"
    }
}

fn render_mobile_html(locale: &str) -> String {
    MOBILE_HTML.replace("__DD_LOCALE__", normalize_locale(locale))
}

async fn verify_token(
    Query(query): Query<VerifyQuery>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let valid = query.token.as_deref() == Some(&state.token);
    if valid {
        Json(serde_json::json!({ "valid": true }))
    } else {
        Json(serde_json::json!({ "valid": false }))
    }
}

async fn wifi_events(
    ws: WebSocketUpgrade,
    Query(query): Query<VerifyQuery>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    if query.token.as_deref() != Some(&state.token) {
        return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
    }

    let rx = state.event_tx.subscribe();
    ws.on_upgrade(move |socket| stream_wifi_events(socket, rx))
        .into_response()
}

async fn stream_wifi_events(mut socket: WebSocket, mut rx: broadcast::Receiver<WifiFileEvent>) {
    let mut heartbeat = tokio::time::interval(std::time::Duration::from_secs(30));
    loop {
        tokio::select! {
            event = rx.recv() => {
                match event {
                    Ok(event) => {
                        let Ok(payload) = serde_json::to_string(&event) else {
                            continue;
                        };
                        if socket.send(Message::Text(payload.into())).await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
            _ = heartbeat.tick() => {
                if socket.send(Message::Ping(Bytes::new())).await.is_err() {
                    break;
                }
            }
        }
    }
}

async fn upload_file(
    State(state): State<AppState>,
    Query(query): Query<VerifyQuery>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    if query.token.as_deref() != Some(&state.token) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "Invalid token" })),
        );
    }

    let mut relative_path: Option<String> = None;
    while let Ok(Some(field)) = multipart.next_field().await {
        let field_name = field.name().unwrap_or("").to_string();
        if field_name == "relativePath" {
            if let Ok(text) = field.text().await {
                relative_path = Some(text);
            }
            continue;
        }

        let file_name = field.file_name().unwrap_or("unknown").to_string();
        let raw_path = relative_path
            .take()
            .filter(|p| !p.trim().is_empty())
            .unwrap_or(file_name);
        let safe_path = sanitize_relative_path(&raw_path);
        let dest = unique_upload_path(&state.upload_dir, &safe_path).await;
        if let Some(parent) = dest.parent() {
            if let Err(e) = tokio::fs::create_dir_all(parent).await {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": e.to_string() })),
                );
            }
        }
        if let Err(e) = write_multipart_field(field, &dest).await {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            );
        }

        let _ = tauri::Emitter::emit(
            &state.app_handle,
            "transfer://file-received",
            dest.to_string_lossy().into_owned(),
        );
        let event_path = dest
            .strip_prefix(&state.upload_dir)
            .map(path_to_slash_string)
            .unwrap_or_else(|_| path_to_slash_string(&safe_path));
        broadcast_wifi_file_event(
            "file.created",
            Some(event_path.clone()),
            query.client_id.clone(),
        );
        push_transfer_history(
            &state,
            TransferHistoryEntry {
                id: nanoid::nanoid!(10),
                entry_type: "upload".to_string(),
                name: dest
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string(),
                path: event_path,
                size: tokio::fs::metadata(&dest)
                    .await
                    .map(|m| m.len())
                    .unwrap_or(0),
                status: "completed".to_string(),
                client_id: query.client_id.clone(),
                checksum: None,
                message: Some("multipart".to_string()),
                timestamp: chrono::Utc::now().timestamp_millis(),
            },
        );
    }

    (StatusCode::OK, Json(serde_json::json!({ "ok": true })))
}

async fn upload_status(
    State(state): State<AppState>,
    Query(query): Query<UploadStatusQuery>,
) -> impl IntoResponse {
    if query.token.as_deref() != Some(&state.token) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "Invalid token" })),
        );
    }

    let partial = partial_upload_path(&state.upload_dir, &query.upload_id);
    let uploaded_bytes = tokio::fs::metadata(partial)
        .await
        .map(|metadata| metadata.len())
        .unwrap_or(0);
    (
        StatusCode::OK,
        Json(serde_json::json!({ "uploadedBytes": uploaded_bytes })),
    )
}

async fn upload_chunk(
    State(state): State<AppState>,
    Query(query): Query<ChunkUploadQuery>,
    body: Bytes,
) -> impl IntoResponse {
    if query.token.as_deref() != Some(&state.token) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "Invalid token" })),
        );
    }
    if has_parent_path_segment(&query.path) || query.offset > query.file_size {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Invalid upload" })),
        );
    }

    let chunk_len = body.len() as u64;
    if query.offset.saturating_add(chunk_len) > query.file_size {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Chunk exceeds declared size" })),
        );
    }

    let partial = partial_upload_path(&state.upload_dir, &query.upload_id);
    if let Some(parent) = partial.parent() {
        if let Err(e) = tokio::fs::create_dir_all(parent).await {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            );
        }
    }

    let mut file = match tokio::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .read(true)
        .open(&partial)
        .await
    {
        Ok(file) => file,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        }
    };

    if let Err(e) = file.seek(SeekFrom::Start(query.offset)).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        );
    }
    if let Err(e) = file.write_all(&body).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        );
    }
    if let Err(e) = file.flush().await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        );
    }
    drop(file);

    let uploaded_bytes = tokio::fs::metadata(&partial)
        .await
        .map(|metadata| metadata.len())
        .unwrap_or(0)
        .min(query.file_size);

    if uploaded_bytes < query.file_size {
        return (
            StatusCode::OK,
            Json(serde_json::json!({
                "ok": true,
                "completed": false,
                "uploadedBytes": uploaded_bytes
            })),
        );
    }

    let safe_path = sanitize_relative_path(&query.path);
    let dest = unique_upload_path(&state.upload_dir, &safe_path).await;
    if let Some(parent) = dest.parent() {
        if let Err(e) = tokio::fs::create_dir_all(parent).await {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            );
        }
    }

    if let Err(e) = tokio::fs::rename(&partial, &dest).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        );
    }

    let metadata = match tokio::fs::metadata(&dest).await {
        Ok(metadata) => metadata,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        }
    };
    if metadata.len() != query.file_size {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": "Size verification failed" })),
        );
    }

    let checksum = hash_file_sha256(dest.clone()).await.ok();
    let _ = tauri::Emitter::emit(
        &state.app_handle,
        "transfer://file-received",
        dest.to_string_lossy().into_owned(),
    );
    let event_path = dest
        .strip_prefix(&state.upload_dir)
        .map(path_to_slash_string)
        .unwrap_or_else(|_| path_to_slash_string(&safe_path));
    broadcast_wifi_file_event(
        "file.created",
        Some(event_path.clone()),
        query.client_id.clone(),
    );
    push_transfer_history(
        &state,
        TransferHistoryEntry {
            id: nanoid::nanoid!(10),
            entry_type: "upload".to_string(),
            name: dest
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string(),
            path: event_path.clone(),
            size: metadata.len(),
            status: "completed".to_string(),
            client_id: query.client_id.clone(),
            checksum: checksum.clone(),
            message: Some("chunked+size-verified".to_string()),
            timestamp: chrono::Utc::now().timestamp_millis(),
        },
    );

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "ok": true,
            "completed": true,
            "uploadedBytes": metadata.len(),
            "path": event_path,
            "checksum": checksum
        })),
    )
}

fn format_access_url(ip: std::net::IpAddr, port: u16) -> String {
    match ip {
        std::net::IpAddr::V4(ip) => format!("http://{}:{}", ip, port),
        std::net::IpAddr::V6(ip) => format!("http://[{}]:{}", ip, port),
    }
}

async fn unique_upload_path(upload_dir: &FsPath, relative_path: &FsPath) -> PathBuf {
    let candidate = upload_dir.join(relative_path);
    if tokio::fs::metadata(&candidate).await.is_err() {
        return candidate;
    }

    let parent = relative_path.parent().unwrap_or_else(|| FsPath::new(""));
    let file_name = relative_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("file");
    let path = FsPath::new(file_name);
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("file");
    let extension = path.extension().and_then(|s| s.to_str());
    for index in 1..1000 {
        let name = match extension {
            Some(extension) if !extension.is_empty() => format!("{stem} ({index}).{extension}"),
            _ => format!("{stem} ({index})"),
        };
        let candidate = upload_dir.join(parent).join(name);
        if tokio::fs::metadata(&candidate).await.is_err() {
            return candidate;
        }
    }
    upload_dir
        .join(parent)
        .join(format!("{}-{}", nanoid::nanoid!(6), file_name))
}

async fn write_multipart_field(
    mut field: axum::extract::multipart::Field<'_>,
    dest: &std::path::Path,
) -> Result<(), std::io::Error> {
    let mut file = tokio::fs::File::create(dest).await?;
    while let Some(chunk) = field
        .chunk()
        .await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?
    {
        file.write_all(&chunk).await?;
    }
    file.flush().await?;
    Ok(())
}

fn partial_upload_path(upload_dir: &FsPath, upload_id: &str) -> PathBuf {
    upload_dir
        .join(PARTIAL_UPLOAD_DIR_NAME)
        .join(format!("{}.part", sanitize_filename(upload_id)))
}

async fn file_response(
    file_path: &FsPath,
    display_name: &str,
    file_size: u64,
    disposition_type: &str,
    content_type: &'static str,
    range: Option<&HeaderValue>,
) -> Result<Response, io::Error> {
    let mut file = tokio::fs::File::open(file_path).await?;
    let range = range.and_then(|value| value.to_str().ok());
    let parsed_range = range.and_then(|value| parse_range_header(value, file_size));
    let (status, start, end) = match parsed_range {
        Some((start, end)) => (StatusCode::PARTIAL_CONTENT, start, end),
        None => (StatusCode::OK, 0, file_size.saturating_sub(1)),
    };
    let content_len = if file_size == 0 { 0 } else { end - start + 1 };
    if start > 0 {
        file.seek(SeekFrom::Start(start)).await?;
    }

    let body = if content_len == 0 {
        Body::empty()
    } else {
        Body::from_stream(ReaderStream::new(file.take(content_len)))
    };
    let disposition = format!(
        "{disposition_type}; filename*=UTF-8''{}",
        percent_encode_header_value(display_name)
    );
    let mut response = Response::new(body);
    *response.status_mut() = status;
    response
        .headers_mut()
        .insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type));
    response
        .headers_mut()
        .insert(header::ACCEPT_RANGES, HeaderValue::from_static("bytes"));
    if let Ok(value) = HeaderValue::from_str(&content_len.to_string()) {
        response.headers_mut().insert(header::CONTENT_LENGTH, value);
    }
    if status == StatusCode::PARTIAL_CONTENT {
        let value = format!("bytes {start}-{end}/{file_size}");
        if let Ok(value) = HeaderValue::from_str(&value) {
            response.headers_mut().insert(header::CONTENT_RANGE, value);
        }
    }
    if let Ok(value) = HeaderValue::from_str(&disposition) {
        response
            .headers_mut()
            .insert(header::CONTENT_DISPOSITION, value);
    }
    Ok(response)
}

fn parse_range_header(value: &str, file_size: u64) -> Option<(u64, u64)> {
    if file_size == 0 {
        return None;
    }
    let range = value.strip_prefix("bytes=")?;
    let (start, end) = range.split_once('-')?;
    if start.is_empty() {
        let suffix = end.parse::<u64>().ok()?.min(file_size);
        return Some((file_size - suffix, file_size - 1));
    }
    let start = start.parse::<u64>().ok()?;
    if start >= file_size {
        return None;
    }
    let end = if end.is_empty() {
        file_size - 1
    } else {
        end.parse::<u64>().ok()?.min(file_size - 1)
    };
    if end < start {
        return None;
    }
    Some((start, end))
}

async fn hash_file_sha256(path: PathBuf) -> Result<String, io::Error> {
    tokio::task::spawn_blocking(move || {
        let mut file = std::fs::File::open(path)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 64 * 1024];
        loop {
            let read = file.read(&mut buffer)?;
            if read == 0 {
                break;
            }
            hasher.update(&buffer[..read]);
        }
        Ok::<_, io::Error>(format!("{:x}", hasher.finalize()))
    })
    .await
    .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?
}

fn push_transfer_history(state: &AppState, entry: TransferHistoryEntry) {
    if let Ok(mut history) = state.history.lock() {
        history.insert(0, entry);
        history.truncate(100);
    }
}

fn parse_batch_paths(paths: &str) -> Vec<String> {
    if let Ok(values) = serde_json::from_str::<Vec<String>>(paths) {
        return values;
    }
    paths
        .split(',')
        .map(str::trim)
        .filter(|path| !path.is_empty())
        .map(ToString::to_string)
        .collect()
}

async fn list_files(
    State(state): State<AppState>,
    Query(query): Query<VerifyQuery>,
) -> impl IntoResponse {
    if query.token.as_deref() != Some(&state.token) {
        return (StatusCode::UNAUTHORIZED, Json(Vec::<FileInfo>::new()));
    }
    if query.path.as_deref().is_some_and(has_parent_path_segment) {
        return (StatusCode::BAD_REQUEST, Json(Vec::<FileInfo>::new()));
    }

    let relative = sanitize_relative_path(query.path.as_deref().unwrap_or(""));
    let current_dir = state.upload_dir.join(&relative);
    let mut files = Vec::new();
    if let Ok(mut entries) = tokio::fs::read_dir(&current_dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            if let Ok(metadata) = entry.metadata().await {
                if let Some(name) = entry.file_name().to_str() {
                    if name == PARTIAL_UPLOAD_DIR_NAME {
                        continue;
                    }
                    let path = append_relative_name(&relative, name);
                    files.push(FileInfo {
                        name: name.to_string(),
                        path,
                        size: if metadata.is_file() {
                            metadata.len()
                        } else {
                            0
                        },
                        is_directory: metadata.is_dir(),
                        modified: metadata
                            .modified()
                            .ok()
                            .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
                            .map(|duration| duration.as_millis() as i64),
                    });
                }
            }
        }
    }

    files.sort_by(|a, b| {
        b.is_directory
            .cmp(&a.is_directory)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    (StatusCode::OK, Json(files))
}

async fn download_file(
    State(state): State<AppState>,
    Query(query): Query<VerifyQuery>,
    Path(path): Path<String>,
    headers: HeaderMap,
) -> Response {
    if query.token.as_deref() != Some(&state.token) {
        return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
    }
    if has_parent_path_segment(&path) {
        return (StatusCode::BAD_REQUEST, "Invalid path").into_response();
    }

    let relative = sanitize_relative_path(&path);
    let file_path = state.upload_dir.join(&relative);

    let metadata = match tokio::fs::metadata(&file_path).await {
        Ok(metadata) => metadata,
        Err(_) => return (StatusCode::NOT_FOUND, "File not found").into_response(),
    };

    if metadata.is_dir() {
        let stream = build_zip_archive_stream(&state.upload_dir, &relative);
        let file_name = zip_file_name(&relative);
        let disposition = format!(
            "attachment; filename*=UTF-8''{}",
            percent_encode_header_value(&file_name)
        );
        let mut response = Response::new(Body::from_stream(stream));
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/zip"),
        );
        if let Ok(value) = HeaderValue::from_str(&disposition) {
            response
                .headers_mut()
                .insert(header::CONTENT_DISPOSITION, value);
        }
        return response;
    }

    let display_name = relative
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("download");

    match file_response(
        &file_path,
        display_name,
        metadata.len(),
        "attachment",
        guess_content_type(display_name),
        headers.get(header::RANGE),
    )
    .await
    {
        Ok(response) => {
            push_transfer_history(
                &state,
                TransferHistoryEntry {
                    id: nanoid::nanoid!(10),
                    entry_type: "download".to_string(),
                    name: display_name.to_string(),
                    path: path_to_slash_string(&relative),
                    size: metadata.len(),
                    status: "started".to_string(),
                    client_id: query.client_id.clone(),
                    checksum: None,
                    message: None,
                    timestamp: chrono::Utc::now().timestamp_millis(),
                },
            );
            response
        }
        Err(_) => (StatusCode::NOT_FOUND, "File not found").into_response(),
    }
}

async fn preview_file(
    State(state): State<AppState>,
    Query(query): Query<VerifyQuery>,
    Path(path): Path<String>,
    headers: HeaderMap,
) -> Response {
    if query.token.as_deref() != Some(&state.token) {
        return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
    }
    if has_parent_path_segment(&path) {
        return (StatusCode::BAD_REQUEST, "Invalid path").into_response();
    }

    let relative = sanitize_relative_path(&path);
    let file_path = state.upload_dir.join(&relative);

    let metadata = match tokio::fs::metadata(&file_path).await {
        Ok(metadata) => metadata,
        Err(_) => return (StatusCode::NOT_FOUND, "File not found").into_response(),
    };

    if metadata.is_dir() {
        return (StatusCode::BAD_REQUEST, "Directories cannot be previewed").into_response();
    }

    let display_name = relative
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("preview");

    match file_response(
        &file_path,
        display_name,
        metadata.len(),
        "inline",
        preview_content_type(display_name),
        headers.get(header::RANGE),
    )
    .await
    {
        Ok(mut response) => {
            response.headers_mut().insert(
                "x-content-type-options",
                HeaderValue::from_static("nosniff"),
            );
            response
        }
        Err(_) => (StatusCode::NOT_FOUND, "File not found").into_response(),
    }
}

async fn download_selected_zip(
    State(state): State<AppState>,
    Query(query): Query<BatchDownloadQuery>,
) -> Response {
    if query.token.as_deref() != Some(&state.token) {
        return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
    }

    let paths = parse_batch_paths(&query.paths);
    if paths.is_empty() || paths.iter().any(|path| has_parent_path_segment(path)) {
        return (StatusCode::BAD_REQUEST, "Invalid paths").into_response();
    }
    let relatives = paths
        .iter()
        .map(|path| sanitize_relative_path(path))
        .filter(|path| !path.as_os_str().is_empty())
        .collect::<Vec<_>>();
    if relatives.is_empty() {
        return (StatusCode::BAD_REQUEST, "Invalid paths").into_response();
    }

    let stream = build_selected_zip_archive_stream(&state.upload_dir, relatives);
    let disposition = format!(
        "attachment; filename*=UTF-8''{}",
        percent_encode_header_value("devicedeck-files.zip")
    );
    let mut response = Response::new(Body::from_stream(stream));
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/zip"),
    );
    if let Ok(value) = HeaderValue::from_str(&disposition) {
        response
            .headers_mut()
            .insert(header::CONTENT_DISPOSITION, value);
    }
    response
}

async fn transfer_history(
    State(state): State<AppState>,
    Query(query): Query<VerifyQuery>,
) -> impl IntoResponse {
    if query.token.as_deref() != Some(&state.token) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(Vec::<TransferHistoryEntry>::new()),
        );
    }

    let entries = state
        .history
        .lock()
        .map(|history| history.clone())
        .unwrap_or_default();
    (StatusCode::OK, Json(entries))
}

async fn delete_file(
    State(state): State<AppState>,
    Query(query): Query<VerifyQuery>,
    Path(path): Path<String>,
) -> impl IntoResponse {
    if query.token.as_deref() != Some(&state.token) {
        return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
    }
    if has_parent_path_segment(&path) {
        return (StatusCode::BAD_REQUEST, "Invalid path").into_response();
    }

    let relative = sanitize_relative_path(&path);
    let file_path = state.upload_dir.join(&relative);

    if !file_path.starts_with(&state.upload_dir) {
        return (StatusCode::BAD_REQUEST, "Invalid path").into_response();
    }

    let result = match tokio::fs::metadata(&file_path).await {
        Ok(metadata) if metadata.is_dir() => tokio::fs::remove_dir_all(&file_path).await,
        Ok(_) => tokio::fs::remove_file(&file_path).await,
        Err(_) => return (StatusCode::NOT_FOUND, "File not found").into_response(),
    };

    match result {
        Ok(_) => {
            broadcast_wifi_file_event(
                "file.deleted",
                Some(path_to_slash_string(&relative)),
                query.client_id.clone(),
            );
            (StatusCode::OK, "Deleted").into_response()
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Delete failed").into_response(),
    }
}

fn sanitize_filename(name: &str) -> String {
    let sanitized: String = name
        .chars()
        .map(|c| {
            if c.is_control()
                || matches!(
                    c,
                    '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' | '\'' | '`'
                )
            {
                '_'
            } else {
                c
            }
        })
        .collect();
    let sanitized = sanitized.trim_matches([' ', '.']).to_string();
    if sanitized.is_empty() {
        "unknown".into()
    } else {
        sanitized
    }
}

fn sanitize_path_component(name: &str) -> Option<String> {
    let sanitized = sanitize_filename(name);
    if sanitized == "unknown" && name.trim().is_empty() {
        None
    } else {
        Some(sanitized)
    }
}

fn has_parent_path_segment(path: &str) -> bool {
    path.split(['/', '\\']).any(|part| part.trim() == "..")
}

fn sanitize_relative_path(path: &str) -> PathBuf {
    let mut safe = PathBuf::new();
    for part in path.split(['/', '\\']) {
        let trimmed = part.trim();
        if trimmed.is_empty() || trimmed == "." || trimmed == ".." {
            continue;
        }
        if let Some(component) = sanitize_path_component(trimmed) {
            safe.push(component);
        }
    }
    safe
}

fn append_relative_name(parent: &FsPath, name: &str) -> String {
    let mut path = parent.to_path_buf();
    path.push(name);
    path_to_slash_string(&path)
}

fn path_to_slash_string(path: &FsPath) -> String {
    path.components()
        .filter_map(|component| component.as_os_str().to_str())
        .collect::<Vec<_>>()
        .join("/")
}

fn zip_file_name(relative: &FsPath) -> String {
    let name = relative
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("folder");
    format!("{}.zip", sanitize_filename(name))
}

fn build_zip_archive_stream(
    upload_dir: &FsPath,
    relative: &FsPath,
) -> ReceiverStream<Result<Bytes, io::Error>> {
    let (sender, receiver) = mpsc::channel::<Result<Bytes, io::Error>>(8);
    let upload_dir = upload_dir.to_path_buf();
    let relative = relative.to_path_buf();
    tokio::task::spawn_blocking(move || {
        let error_sender = sender.clone();
        if let Err(e) = build_zip_archive_sync(&upload_dir, &relative, ChannelWriter { sender }) {
            let _ = error_sender.blocking_send(Err(io::Error::new(io::ErrorKind::Other, e)));
        }
    });
    ReceiverStream::new(receiver)
}

fn build_selected_zip_archive_stream(
    upload_dir: &FsPath,
    relatives: Vec<PathBuf>,
) -> ReceiverStream<Result<Bytes, io::Error>> {
    let (sender, receiver) = mpsc::channel::<Result<Bytes, io::Error>>(8);
    let upload_dir = upload_dir.to_path_buf();
    tokio::task::spawn_blocking(move || {
        let error_sender = sender.clone();
        if let Err(e) =
            build_selected_zip_archive_sync(&upload_dir, &relatives, ChannelWriter { sender })
        {
            let _ = error_sender.blocking_send(Err(io::Error::new(io::ErrorKind::Other, e)));
        }
    });
    ReceiverStream::new(receiver)
}

struct ChannelWriter {
    sender: mpsc::Sender<Result<Bytes, io::Error>>,
}

impl Write for ChannelWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }
        self.sender
            .blocking_send(Ok(Bytes::copy_from_slice(buf)))
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "download disconnected"))?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn build_zip_archive_sync<W>(
    upload_dir: &FsPath,
    relative: &FsPath,
    writer: W,
) -> Result<(), String>
where
    W: Write,
{
    let source_dir = upload_dir.join(relative);
    let root_name = source_dir
        .file_name()
        .and_then(|s| s.to_str())
        .map(sanitize_filename)
        .unwrap_or_else(|| "folder".to_string());

    let mut zip = zip::ZipWriter::new_stream(writer);
    let options =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

    add_zip_directory(&mut zip, &source_dir, &root_name, options)?;
    zip.finish().map_err(|e| e.to_string())?;
    Ok(())
}

fn build_selected_zip_archive_sync<W>(
    upload_dir: &FsPath,
    relatives: &[PathBuf],
    writer: W,
) -> Result<(), String>
where
    W: Write,
{
    let mut zip = zip::ZipWriter::new_stream(writer);
    let options =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

    for relative in relatives {
        let source = upload_dir.join(relative);
        let metadata = match std::fs::metadata(&source) {
            Ok(metadata) => metadata,
            Err(_) => continue,
        };
        let zip_name = path_to_slash_string(relative);
        if metadata.is_dir() {
            add_zip_directory(&mut zip, &source, &zip_name, options)?;
        } else if metadata.is_file() {
            zip.start_file(&zip_name, options)
                .map_err(|e| e.to_string())?;
            let mut file = std::fs::File::open(&source).map_err(|e| e.to_string())?;
            std::io::copy(&mut file, &mut zip).map_err(|e| e.to_string())?;
        }
    }

    zip.finish().map_err(|e| e.to_string())?;
    Ok(())
}

fn add_zip_directory<W>(
    zip: &mut zip::ZipWriter<W>,
    dir: &FsPath,
    zip_path: &str,
    options: zip::write::SimpleFileOptions,
) -> Result<(), String>
where
    W: Write + Seek,
{
    let dir_name = if zip_path.ends_with('/') {
        zip_path.to_string()
    } else {
        format!("{zip_path}/")
    };
    zip.add_directory(&dir_name, options)
        .map_err(|e| e.to_string())?;

    let entries = std::fs::read_dir(dir).map_err(|e| e.to_string())?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let metadata = entry.metadata().map_err(|e| e.to_string())?;
        let name = entry
            .file_name()
            .to_str()
            .map(sanitize_filename)
            .unwrap_or_else(|| "unknown".to_string());
        let child_zip_path = format!("{zip_path}/{name}");

        if metadata.is_dir() {
            add_zip_directory(zip, &path, &child_zip_path, options)?;
        } else if metadata.is_file() {
            zip.start_file(&child_zip_path, options)
                .map_err(|e| e.to_string())?;
            let mut file = std::fs::File::open(&path).map_err(|e| e.to_string())?;
            std::io::copy(&mut file, zip).map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

fn percent_encode_header_value(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.as_bytes() {
        let ch = *byte as char;
        if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | '~') {
            encoded.push(ch);
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    encoded
}

fn guess_content_type(name: &str) -> &'static str {
    match file_extension(name).as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        "ico" => "image/x-icon",
        "avif" => "image/avif",
        "pdf" => "application/pdf",
        "zip" => "application/zip",
        "rar" => "application/vnd.rar",
        "7z" => "application/x-7z-compressed",
        "tar" => "application/x-tar",
        "gz" => "application/gzip",
        "apk" => "application/vnd.android.package-archive",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "mov" => "video/quicktime",
        "m4v" => "video/x-m4v",
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "ogg" => "audio/ogg",
        "m4a" => "audio/mp4",
        "flac" => "audio/flac",
        "txt" | "log" | "md" | "csv" | "json" | "jsonl" | "js" | "ts" | "jsx" | "tsx" | "css"
        | "html" | "htm" | "xml" | "svg" | "yaml" | "yml" | "toml" | "rs" | "py" | "go"
        | "java" | "c" | "cpp" | "h" | "hpp" | "sh" | "bat" | "ps1" | "sql" => {
            "text/plain; charset=utf-8"
        }
        _ => "application/octet-stream",
    }
}

fn preview_content_type(name: &str) -> &'static str {
    match file_extension(name).as_str() {
        "html" | "htm" | "svg" => "text/plain; charset=utf-8",
        _ => guess_content_type(name),
    }
}

fn file_extension(name: &str) -> String {
    name.rsplit('.').next().unwrap_or("").to_ascii_lowercase()
}

const MOBILE_HTML: &str = r##"
<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width,initial-scale=1,maximum-scale=1,user-scalable=no">
<title>DeviceDeck 文件传输</title>
<script>
(function() {
  try {
    const saved = localStorage.getItem('devicedeck-wifi-theme');
    const prefersLight = window.matchMedia && window.matchMedia('(prefers-color-scheme: light)').matches;
    document.documentElement.dataset.theme = saved || (prefersLight ? 'light' : 'dark');
  } catch {
    document.documentElement.dataset.theme = 'dark';
  }
})();
</script>
<style>
:root {
  color-scheme: dark;
  --bg: #050507;
  --surface: #0c0c10;
  --surface-2: #141418;
  --surface-3: #1c1c22;
  --border: #232329;
  --border-hover: #2e2e38;
  --fg: #f0f0f2;
  --fg-2: #a8a8b2;
  --fg-3: #62626c;
  --accent: #00d992;
  --accent-soft: rgba(0,217,146,0.1);
  --accent-border: rgba(0,217,146,0.3);
  --danger: #fb565b;
  --danger-soft: rgba(251,86,91,0.1);
  --success: #00d992;
  --warning: #ffb224;
  --warning-soft: rgba(255,178,36,0.1);
  --header-bg: rgba(5,5,7,.85);
  --auth-btn-fg: #050507;
  --shadow: rgba(0,0,0,.4);
  --r: 12px;
  --r-sm: 8px;
  --r-xs: 6px;
}

html[data-theme="light"] {
  color-scheme: light;
  --bg: #f6f7f9;
  --surface: #ffffff;
  --surface-2: #eef1f4;
  --surface-3: #e3e8ed;
  --border: #d9e0e6;
  --border-hover: #c3cdd6;
  --fg: #12161c;
  --fg-2: #4c5966;
  --fg-3: #7d8a96;
  --accent: #008c62;
  --accent-soft: rgba(0,140,98,0.1);
  --accent-border: rgba(0,140,98,0.28);
  --danger: #c92f43;
  --danger-soft: rgba(201,47,67,0.1);
  --success: #008c62;
  --warning: #9a6400;
  --warning-soft: rgba(154,100,0,0.12);
  --header-bg: rgba(246,247,249,.88);
  --auth-btn-fg: #ffffff;
  --shadow: rgba(16,24,40,.14);
}

*{box-sizing:border-box;margin:0;padding:0}
html{-webkit-tap-highlight-color:transparent}
body{
  font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',system-ui,sans-serif;
  background:var(--bg);color:var(--fg);
  min-height:100vh;min-height:100dvh;
  -webkit-font-smoothing:antialiased;
  overflow-x:hidden;
}

.screen{display:none;min-height:100vh;min-height:100dvh}
.screen.active{display:flex}

#auth{
  flex-direction:column;align-items:center;justify-content:center;
  padding:32px 24px;gap:0;
}
.auth-logo{
  width:64px;height:64px;border-radius:16px;
  background:var(--surface-2);border:1px solid var(--border);
  display:flex;align-items:center;justify-content:center;
  margin-bottom:20px;
}
.auth-logo svg{width:32px;height:32px;color:var(--accent)}
.auth-title{font-size:22px;font-weight:700;letter-spacing:-0.3px;margin-bottom:4px}
.auth-sub{font-size:14px;color:var(--fg-3);margin-bottom:36px}
.auth-form{width:100%;max-width:340px}
.auth-label{font-size:13px;color:var(--fg-2);margin-bottom:8px;display:block}
.auth-input-wrap{display:flex;gap:8px;margin-bottom:12px}
.auth-input{
  flex:1;padding:14px 16px;border-radius:var(--r-sm);
  background:var(--surface-2);border:1px solid var(--border);
  color:var(--fg);font-size:20px;font-weight:600;
  font-family:'SF Mono',SFMono-Regular,ui-monospace,Menlo,monospace;
  letter-spacing:6px;text-align:center;outline:none;
  transition:border-color .2s;
}
.auth-input:focus{border-color:var(--accent)}
.auth-input::placeholder{letter-spacing:2px;font-size:14px;font-weight:400;color:var(--fg-3)}
.auth-btn{
  width:100%;padding:14px;border:none;border-radius:var(--r-sm);
  background:var(--accent);color:var(--auth-btn-fg);font-size:15px;font-weight:600;
  cursor:pointer;transition:all .2s;display:flex;align-items:center;
  justify-content:center;gap:8px;
}
.auth-btn:hover{opacity:.9}
.auth-btn:disabled{opacity:.4;cursor:not-allowed}
.auth-btn svg{width:18px;height:18px}
.auth-error{
  margin-top:12px;font-size:13px;color:var(--danger);
  text-align:center;min-height:20px;
}
.auth-error.shake{animation:shake .4s ease}
.auth-hint{margin-top:24px;font-size:12px;color:var(--fg-3);text-align:center;line-height:1.6}

@keyframes shake{
  0%,100%{transform:translateX(0)}
  20%,60%{transform:translateX(-6px)}
  40%,80%{transform:translateX(6px)}
}

#main{flex-direction:column}

.header{
  position:sticky;top:0;z-index:100;
  padding:12px 16px;
  background:var(--header-bg);
  backdrop-filter:blur(16px);-webkit-backdrop-filter:blur(16px);
  border-bottom:1px solid var(--border);
  display:flex;align-items:center;justify-content:space-between;
}
.header-left{display:flex;align-items:center;gap:10px}
.header-logo{font-size:16px;font-weight:700;letter-spacing:-0.3px}
.header-status{
  display:flex;align-items:center;gap:6px;
  font-size:12px;color:var(--fg-3);
}
.status-dot{
  width:6px;height:6px;border-radius:50%;background:var(--accent);
  box-shadow:0 0 6px var(--accent);
  animation:pulse-dot 2s ease-in-out infinite;
}
.status-dot.disconnected{background:var(--danger);box-shadow:0 0 6px var(--danger)}
.status-dot.reconnecting{background:var(--warning);box-shadow:0 0 6px var(--warning);animation:pulse-dot .8s ease-in-out infinite}
@keyframes pulse-dot{
  0%,100%{opacity:1}
  50%{opacity:.4}
}
.header-actions{display:flex;gap:4px}
.icon-btn{
  width:40px;height:40px;border:none;border-radius:var(--r-xs);
  background:transparent;color:var(--fg-2);cursor:pointer;
  display:flex;align-items:center;justify-content:center;
  transition:all .15s;-webkit-user-select:none;user-select:none;
}
.icon-btn:hover{background:var(--surface-3);color:var(--fg)}
.icon-btn:active{transform:scale(.92)}
.icon-btn svg{width:18px;height:18px}
.auth-theme-toggle{
  position:fixed;top:16px;right:16px;z-index:120;
  background:var(--surface);border:1px solid var(--border);
}
.theme-icon{display:none}
html[data-theme="dark"] .theme-icon-sun{display:block}
html[data-theme="light"] .theme-icon-moon{display:block}

.content{
  padding:16px;padding-bottom:32px;
  display:flex;flex-direction:column;gap:16px;
  max-width:640px;margin:0 auto;width:100%;
}

.upload-zone{
  border:2px dashed var(--border);border-radius:var(--r);
  padding:32px 20px;text-align:center;
  cursor:pointer;transition:all .25s;
  background:var(--surface);
}
.upload-zone:hover,.upload-zone.dragover{
  border-color:var(--accent-border);
  background:var(--accent-soft);
}
.upload-zone.dragover{
  border-style:solid;
  border-color:var(--accent);
  box-shadow:0 0 24px rgba(0,217,146,0.08);
}
.upload-zone.disabled{opacity:.4;pointer-events:none}
.upload-zone svg{width:40px;height:40px;color:var(--fg-3);margin-bottom:12px}
.upload-zone-title{font-size:15px;font-weight:500;margin-bottom:4px;color:var(--fg)}
.upload-zone-sub{font-size:13px;color:var(--fg-3);margin-bottom:20px}
.upload-actions{display:flex;gap:8px;justify-content:center}
.upload-action-btn{
  padding:12px 20px;border-radius:var(--r-sm);border:1px solid var(--border);
  background:var(--surface-2);color:var(--fg-2);font-size:13px;font-weight:500;
  cursor:pointer;transition:all .15s;display:flex;align-items:center;gap:6px;
  -webkit-user-select:none;user-select:none;
}
.upload-action-btn:hover{border-color:var(--border-hover);color:var(--fg)}
.upload-action-btn.unsupported{
  opacity:.55;
  cursor:not-allowed;
}
.upload-action-btn.unsupported:hover{
  border-color:var(--border);
  color:var(--fg-2);
}
.upload-action-btn svg{width:16px;height:16px}

.queue-section{
  background:var(--surface);border:1px solid var(--border);
  border-radius:var(--r);overflow:hidden;
}
.queue-header{
  padding:14px 16px;display:flex;align-items:center;
  justify-content:space-between;border-bottom:1px solid var(--border);
}
.queue-title{font-size:14px;font-weight:600;display:flex;align-items:center;gap:8px}
.queue-title .count{
  font-size:11px;font-weight:600;
  background:var(--accent-soft);color:var(--accent);
  padding:2px 8px;border-radius:99px;
}
.queue-progress-summary{
  font-size:11px;color:var(--fg-3);font-weight:500;
}
.queue-header-right{display:flex;align-items:center;gap:12px}
.queue-clear{
  font-size:12px;color:var(--fg-3);background:none;border:none;
  cursor:pointer;transition:color .15s;
}
.queue-clear:hover{color:var(--danger)}
.queue-list{max-height:360px;overflow-y:auto}

.queue-item{
  padding:12px 16px;display:flex;align-items:center;gap:10px;
  border-bottom:1px solid var(--border);
}
.queue-item:last-child{border-bottom:none}
.queue-item.entering{animation:slideIn .2s ease}
@keyframes slideIn{from{opacity:0;transform:translateY(-8px)}to{opacity:1;transform:translateY(0)}}

.queue-item-icon{
  width:36px;height:36px;border-radius:var(--r-xs);
  background:var(--surface-3);display:flex;align-items:center;justify-content:center;
  flex-shrink:0;
}
.queue-item-icon svg{width:18px;height:18px}
.queue-item-icon.folder{background:rgba(56,189,248,0.1);color:#38bdf8}
.queue-item-info{flex:1;min-width:0}
.queue-item-name{
  font-size:13px;font-weight:500;
  overflow:hidden;text-overflow:ellipsis;white-space:nowrap;
}
.queue-item-meta{font-size:11px;color:var(--fg-3);margin-top:2px;display:flex;align-items:center;gap:6px}
.queue-item-status{
  font-size:11px;font-weight:600;padding:2px 6px;
  border-radius:4px;flex-shrink:0;
}
.queue-item-status.uploading{color:var(--accent);background:var(--accent-soft)}
.queue-item-status.waiting{color:var(--fg-3);background:var(--surface-3)}
.queue-item-status.paused{color:var(--warning);background:var(--warning-soft)}
.queue-item-status.done{color:var(--success);background:rgba(0,217,146,0.08)}
.queue-item-status.error{color:var(--danger);background:var(--danger-soft)}

.queue-progress{
  height:3px;background:var(--surface-3);border-radius:2px;
  margin-top:6px;overflow:hidden;
}
.queue-progress-bar{
  height:100%;background:var(--accent);border-radius:2px;
  transition:width .15s ease;width:0;
}

.queue-item-action{
  width:36px;height:36px;border:none;border-radius:var(--r-xs);
  background:transparent;color:var(--fg-3);cursor:pointer;
  display:flex;align-items:center;justify-content:center;
  transition:all .15s;flex-shrink:0;
  -webkit-user-select:none;user-select:none;
}
.queue-item-action:hover{background:var(--danger-soft);color:var(--danger)}
.queue-item-action:active{transform:scale(.9)}
.queue-item-action svg{width:14px;height:14px}
.queue-item-action.retry:hover{background:var(--accent-soft);color:var(--accent)}
.queue-item-actions,.queue-folder-actions{display:flex;align-items:center;gap:4px;flex-shrink:0}
.queue-item-action.pause:hover{background:var(--warning-soft);color:var(--warning)}
.queue-item-action.resume:hover{background:var(--accent-soft);color:var(--accent)}
.queue-folder{
  border-bottom:1px solid var(--border);
}
.queue-folder:last-child{border-bottom:none}
.queue-folder-head{
  padding:10px 16px;display:flex;align-items:center;gap:10px;
  cursor:pointer;transition:background .15s;
}
.queue-folder-head:active{background:var(--surface-2)}
.queue-folder-toggle{
  width:24px;height:24px;border:none;border-radius:var(--r-xs);
  background:transparent;color:var(--fg-3);display:flex;align-items:center;justify-content:center;
  flex-shrink:0;cursor:pointer;
}
.queue-folder-children{
  border-top:1px solid var(--border);
  background:var(--surface-2);
}
.queue-item-child{padding-left:42px}

.queue-footer{padding:12px 16px;border-top:1px solid var(--border)}
.upload-all-btn{
  width:100%;padding:12px;border:none;border-radius:var(--r-sm);
  background:var(--accent);color:#050507;font-size:14px;font-weight:600;
  cursor:pointer;transition:all .2s;
  display:flex;align-items:center;justify-content:center;gap:8px;
}
.upload-all-btn:hover{opacity:.9}
.upload-all-btn:disabled{opacity:.35;cursor:not-allowed}
.upload-all-btn svg{width:16px;height:16px}

.files-section{
  background:var(--surface);border:1px solid var(--border);
  border-radius:var(--r);overflow:hidden;
}
.files-header{
  padding:14px 16px;display:flex;align-items:center;
  justify-content:space-between;border-bottom:1px solid var(--border);
}
.files-title{font-size:14px;font-weight:600;display:flex;align-items:center;gap:8px}
.files-title .count{
  font-size:11px;font-weight:600;
  background:var(--surface-3);color:var(--fg-2);
  padding:2px 8px;border-radius:99px;
}
.files-header-right{display:flex;align-items:center;gap:8px}
.files-download-all{
  font-size:12px;color:var(--accent);background:none;border:none;
  cursor:pointer;transition:color .15s;display:flex;align-items:center;gap:4px;
}
.files-download-all:hover{opacity:.8}
.files-download-all svg{width:14px;height:14px}
.files-refresh{
  width:32px;height:32px;border:none;border-radius:var(--r-xs);
  background:transparent;color:var(--fg-3);cursor:pointer;
  display:flex;align-items:center;justify-content:center;
  transition:all .15s;
}
.files-refresh:hover{background:var(--surface-3);color:var(--fg)}
.files-refresh.loading svg{animation:spin .8s linear infinite}
.icon-btn.loading svg{animation:spin .8s linear infinite}
.files-refresh svg{width:16px;height:16px}
@keyframes spin{to{transform:rotate(360deg)}}

.files-tools{
  padding:12px;border-bottom:1px solid var(--border);
  display:grid;grid-template-columns:minmax(0,1fr) 132px 122px;gap:8px;
  background:var(--surface);
}
.files-search,.files-select{
  width:100%;height:36px;border:1px solid var(--border);border-radius:var(--r-xs);
  background:var(--surface-2);color:var(--fg);font-size:12px;font-weight:500;
  outline:none;min-width:0;transition:border-color .15s,background .15s,color .15s;
}
.files-search{padding:0 12px}
.files-select{display:none}
.files-search::placeholder{color:var(--fg-3)}
.files-search:focus{
  border-color:var(--accent-border);
  background-color:var(--surface-3);
  color:var(--fg);
}
.files-search:hover{border-color:var(--border-hover)}
.dd-lite{position:relative;display:block;min-width:0}
.dd-lite-trigger{
  width:100%;height:36px;border:1px solid var(--border);border-radius:var(--r-xs);
  background:var(--surface-2);color:var(--fg);font-size:12px;font-weight:500;
  display:flex;align-items:center;justify-content:space-between;gap:8px;
  padding:0 10px;cursor:pointer;transition:border-color .15s,box-shadow .15s,background .15s;
}
.dd-lite-trigger:hover{border-color:var(--border-hover)}
.dd-lite.open .dd-lite-trigger,
.dd-lite-trigger:focus{outline:none;border-color:var(--accent);box-shadow:0 0 0 2px var(--accent-soft)}
.dd-lite-label{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.dd-lite-chevron{
  width:14px;height:14px;color:var(--fg-3);flex-shrink:0;
  transition:transform .15s;display:flex;align-items:center;justify-content:center;
}
.dd-lite.open .dd-lite-chevron{transform:rotate(180deg)}
.dd-lite-panel{
  position:fixed;z-index:9999;
  max-height:220px;overflow-y:auto;padding:4px;
  border:1px solid var(--border);border-radius:var(--r-sm);
  background:var(--surface);box-shadow:0 8px 24px var(--shadow);
  animation:ddIn .12s ease-out;
}
.dd-lite-option{
  min-height:30px;padding:6px 8px;border-radius:var(--r-xs);
  color:var(--fg);font-size:12px;display:flex;align-items:center;justify-content:space-between;gap:8px;
  cursor:pointer;transition:background .1s,color .1s;
}
.dd-lite-option:hover,.dd-lite-option.highlight{background:var(--surface-3)}
.dd-lite-option.active{color:var(--accent);font-weight:600}
.dd-lite-check{color:var(--accent);font-size:12px;line-height:1}
@keyframes ddIn{from{opacity:0;transform:translateY(-4px)}to{opacity:1;transform:translateY(0)}}
.files-bulk{
  padding:8px 12px;border-bottom:1px solid var(--border);
  display:none;align-items:center;justify-content:space-between;gap:8px;
  font-size:12px;color:var(--fg-2);
}
.files-bulk.show{display:flex}
.files-bulk-actions{display:flex;gap:6px;flex-shrink:0}
.mini-action{
  border:1px solid var(--border);border-radius:var(--r-xs);
  background:var(--surface-2);color:var(--fg-2);height:30px;padding:0 10px;
  font-size:12px;cursor:pointer;
}
.mini-action:hover{color:var(--fg);border-color:var(--border-hover)}
.mini-action.danger{color:var(--danger)}
.files-list{max-height:50vh;overflow-y:auto}

.file-item{
  padding:12px 16px;display:flex;align-items:center;gap:10px;
  border-bottom:1px solid var(--border);
  transition:background .15s;
}
.file-item:last-child{border-bottom:none}
.file-item:active{background:var(--surface-2)}
.file-check{
  width:18px;height:18px;accent-color:var(--accent);flex-shrink:0;
}

.file-icon{
  width:36px;height:36px;border-radius:var(--r-xs);
  display:flex;align-items:center;justify-content:center;
  flex-shrink:0;
}
.file-icon svg{width:18px;height:18px}
.file-icon.img{background:rgba(232,121,249,0.1);color:#e879f9}
.file-icon.video{background:rgba(251,146,60,0.1);color:#fb923c}
.file-icon.audio{background:rgba(56,189,248,0.1);color:#38bdf8}
.file-icon.doc{background:rgba(0,217,146,0.1);color:#00d992}
.file-icon.archive{background:rgba(251,191,36,0.1);color:#fbbf24}
.file-icon.code{background:rgba(167,139,250,0.1);color:#a78bfa}
.file-icon.folder{background:rgba(56,189,248,0.1);color:#38bdf8}
.file-icon.default{background:var(--surface-3);color:var(--fg-3)}
.folder-toggle,
.folder-toggle-placeholder{
  width:24px;height:24px;border:none;border-radius:var(--r-xs);
  background:transparent;color:var(--fg-3);display:flex;align-items:center;justify-content:center;
  flex-shrink:0;
}
.folder-toggle{cursor:pointer}
.folder-toggle:active{background:var(--surface-3)}

.file-info{flex:1;min-width:0}
.file-name{font-size:13px;font-weight:500;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.file-size{font-size:11px;color:var(--fg-3);margin-top:2px;
  font-family:'SF Mono',SFMono-Regular,ui-monospace,Menlo,monospace;
}
.file-dl{
  width:40px;height:40px;border:none;border-radius:var(--r-xs);
  background:var(--surface-3);color:var(--fg-2);cursor:pointer;
  display:flex;align-items:center;justify-content:center;
  transition:all .15s;flex-shrink:0;
  -webkit-user-select:none;user-select:none;
}
.file-dl:hover{background:var(--accent-soft);color:var(--accent)}
.file-dl:active{transform:scale(.92)}
.file-dl svg{width:16px;height:16px}
.file-del{
  width:40px;height:40px;border:none;border-radius:var(--r-xs);
  background:transparent;color:var(--fg-3);cursor:pointer;
  display:flex;align-items:center;justify-content:center;
  transition:all .15s;flex-shrink:0;
  -webkit-user-select:none;user-select:none;
}
.file-del:hover{background:var(--danger-soft);color:var(--danger)}
.file-del:active{transform:scale(.92)}
.file-del svg{width:16px;height:16px}
.files-clear-all{
  font-size:12px;color:var(--danger);background:none;border:none;
  cursor:pointer;transition:color .15s;display:flex;align-items:center;gap:4px;
}
.files-clear-all:hover{opacity:.8}
.files-clear-all svg{width:14px;height:14px}

.history-section{
  background:var(--surface);border:1px solid var(--border);
  border-radius:var(--r);overflow:hidden;
}
.history-header{
  padding:14px 16px;display:flex;align-items:center;justify-content:space-between;
  border-bottom:1px solid var(--border);
}
.history-title{font-size:14px;font-weight:600}
.history-list{max-height:220px;overflow-y:auto}
.history-item{
  padding:10px 16px;border-bottom:1px solid var(--border);
  display:flex;align-items:center;gap:10px;
}
.history-item:last-child{border-bottom:none}
.history-info{flex:1;min-width:0}
.history-name{font-size:12px;font-weight:500;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.history-meta{font-size:11px;color:var(--fg-3);margin-top:2px}
.history-badge{
  font-size:11px;font-weight:600;padding:2px 6px;border-radius:4px;
  background:var(--surface-3);color:var(--fg-2);flex-shrink:0;
}
.history-badge.upload{color:var(--accent);background:var(--accent-soft)}
.history-badge.download{color:#38bdf8;background:rgba(56,189,248,0.1)}

.empty-state{padding:40px 20px;text-align:center}
.empty-state svg{width:40px;height:40px;color:var(--fg-3);margin-bottom:12px;opacity:.5}
.empty-state p{font-size:13px;color:var(--fg-3)}

.toast{
  position:fixed;top:16px;left:16px;right:16px;z-index:999;
  padding:12px 16px;border-radius:var(--r-sm);
  font-size:13px;font-weight:500;
  display:flex;align-items:center;gap:8px;
  transform:translateY(-120%);transition:transform .3s cubic-bezier(.4,0,.2,1);
  box-shadow:0 8px 32px var(--shadow);
}
.toast.show{transform:translateY(0)}
.toast.success{background:#0a2e1e;color:var(--accent);border:1px solid rgba(0,217,146,0.2)}
.toast.error{background:#2e0a0e;color:var(--danger);border:1px solid rgba(251,86,91,0.2)}
.toast.info{background:var(--surface-3);color:var(--fg);border:1px solid var(--border)}
.toast.warning{background:#2e1f0a;color:var(--warning);border:1px solid rgba(255,178,36,0.2)}
html[data-theme="light"] .toast.success{background:#edfdf6;border-color:rgba(0,140,98,0.24)}
html[data-theme="light"] .toast.error{background:#fff1f3;border-color:rgba(201,47,67,0.2)}
html[data-theme="light"] .toast.warning{background:#fff7e6;border-color:rgba(154,100,0,0.2)}
.toast svg{width:16px;height:16px;flex-shrink:0}

.modal-overlay{
  position:fixed;inset:0;z-index:200;
  background:rgba(0,0,0,.6);
  display:flex;align-items:center;justify-content:center;
  padding:24px;
  opacity:0;transition:opacity .2s;
  pointer-events:none;
}
.modal-overlay.show{opacity:1;pointer-events:auto}
.modal{
  background:var(--surface);border:1px solid var(--border);
  border-radius:var(--r);padding:24px;
  max-width:320px;width:100%;
  transform:scale(.95);transition:transform .2s;
}
.modal-overlay.show .modal{transform:scale(1)}
.modal-title{font-size:16px;font-weight:600;margin-bottom:8px}
.modal-text{font-size:13px;color:var(--fg-2);line-height:1.5;margin-bottom:20px}
.modal-actions{display:flex;gap:8px}
.modal-btn{
  flex:1;padding:10px;border-radius:var(--r-sm);border:none;
  font-size:13px;font-weight:600;cursor:pointer;transition:all .15s;
}
.modal-btn.cancel{background:var(--surface-3);color:var(--fg-2)}
.modal-btn.cancel:hover{background:var(--border)}
.modal-btn.danger{background:var(--danger);color:#fff}
.modal-btn.danger:hover{opacity:.9}

.preview-overlay{
  position:fixed;inset:0;z-index:220;
  background:rgba(0,0,0,.72);
  display:flex;align-items:flex-end;justify-content:center;
  opacity:0;pointer-events:none;
  transition:opacity .2s;
}
.preview-overlay.show{opacity:1;pointer-events:auto}
.preview-panel{
  width:100%;max-width:840px;height:min(86vh,720px);
  background:var(--surface);border:1px solid var(--border);
  border-radius:18px 18px 0 0;
  box-shadow:0 -16px 48px var(--shadow);
  display:flex;flex-direction:column;overflow:hidden;
  transform:translateY(24px);transition:transform .2s;
}
.preview-overlay.show .preview-panel{transform:translateY(0)}
.preview-header{
  padding:12px 14px;border-bottom:1px solid var(--border);
  display:flex;align-items:center;gap:10px;min-height:64px;
}
.preview-info{flex:1;min-width:0}
.preview-name{
  font-size:14px;font-weight:600;
  overflow:hidden;text-overflow:ellipsis;white-space:nowrap;
}
.preview-meta{
  margin-top:2px;font-size:11px;color:var(--fg-3);
  font-family:'SF Mono',SFMono-Regular,ui-monospace,Menlo,monospace;
}
.preview-actions{display:flex;align-items:center;gap:4px;flex-shrink:0}
.preview-body{
  flex:1;min-height:0;background:var(--bg);
  display:flex;align-items:center;justify-content:center;
  overflow:auto;
}
.preview-media{
  max-width:100%;max-height:100%;
  display:block;
}
.preview-video{width:100%;height:100%;background:#000}
.preview-audio{width:min(520px,calc(100% - 32px))}
.preview-frame{width:100%;height:100%;border:0;background:#fff}
.preview-text{
  align-self:stretch;width:100%;min-height:100%;
  padding:16px;margin:0;overflow:auto;
  color:var(--fg);background:var(--bg);
  font:12px/1.6 'SF Mono',SFMono-Regular,ui-monospace,Menlo,monospace;
  white-space:pre-wrap;word-break:break-word;
}
.preview-state{
  padding:24px;text-align:center;color:var(--fg-3);
  font-size:13px;line-height:1.6;
}
.preview-state svg{
  width:36px;height:36px;margin-bottom:10px;color:var(--fg-3);opacity:.65;
}

.auth-btn:not(:disabled):active,
.upload-action-btn:active,
.upload-all-btn:not(:disabled):active,
.queue-clear:active,
.queue-item-action:active,
.files-download-all:active,
.files-clear-all:active,
.files-refresh:active,
.preview-actions .icon-btn:active,
.modal-btn:active {
  transform:scale(.92);
}

.offline-bar{
  background:var(--danger-soft);border-bottom:1px solid rgba(251,86,91,0.2);
  padding:8px 16px;text-align:center;font-size:12px;color:var(--danger);
  display:none;
}
.offline-bar.show{display:block}
.offline-bar.reconnecting{
  background:var(--warning-soft);border-bottom-color:rgba(255,178,36,0.2);
  color:var(--warning);
}

::-webkit-scrollbar{width:4px}
::-webkit-scrollbar-track{background:transparent}
::-webkit-scrollbar-thumb{background:var(--border);border-radius:2px}

.fade-in{animation:fadeIn .3s ease}
@keyframes fadeIn{from{opacity:0}to{opacity:1}}

@media(min-width:768px){
  .content{padding:24px 32px;gap:20px}
  .upload-zone{padding:48px 32px}
  .queue-list{max-height:480px}
  .files-list{max-height:60vh}
  .preview-overlay{align-items:center;padding:24px}
  .preview-panel{border-radius:var(--r);height:min(82vh,760px)}
}
@media(min-width:1024px){
  .content{max-width:720px;padding:32px 40px}
}
@media(max-width:520px){
  .files-tools{grid-template-columns:1fr 1fr;gap:8px}
  .files-search{grid-column:1 / -1}
}
</style>
</head>
<body>

<div id="auth" class="screen active">
  <button class="icon-btn auth-theme-toggle" id="auth-theme-btn" title="切换主题">
    <svg class="theme-icon theme-icon-sun" xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="4"/><path d="M12 2v2"/><path d="M12 20v2"/><path d="m4.93 4.93 1.41 1.41"/><path d="m17.66 17.66 1.41 1.41"/><path d="M2 12h2"/><path d="M20 12h2"/><path d="m6.34 17.66-1.41 1.41"/><path d="m19.07 4.93-1.41 1.41"/></svg>
    <svg class="theme-icon theme-icon-moon" xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20.985 12.486a9 9 0 1 1-9.473-9.472c.405-.022.617.46.402.803a6 6 0 0 0 8.268 8.268c.344-.215.825-.004.803.401"/></svg>
  </button>
  <div class="auth-logo">
    <svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 20h.01"/><path d="M2 8.82a15 15 0 0 1 20 0"/><path d="M5 12.859a10 10 0 0 1 14 0"/><path d="M8.5 16.429a5 5 0 0 1 7 0"/></svg>
  </div>
  <div class="auth-title">DeviceDeck</div>
  <div class="auth-sub">WiFi 文件传输</div>
  <div class="auth-form">
    <label class="auth-label">输入电脑端显示的 8 位连接码</label>
    <div class="auth-input-wrap">
      <input type="text" id="token-input" class="auth-input" placeholder="输入连接码" maxlength="8" autocomplete="off" autocorrect="off" autocapitalize="off" spellcheck="false">
    </div>
    <button id="verify-btn" class="auth-btn" disabled>
      <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M15 3h4a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2h-4"/><polyline points="10 17 15 12 10 7"/><line x1="15" y1="12" x2="3" y2="12"/></svg>
      连接
    </button>
    <div id="auth-error" class="auth-error"></div>
  </div>
  <div class="auth-hint">
    在电脑上打开 DeviceDeck 并启动<br>WiFi 传输以获取连接码
  </div>
</div>

<div id="main" class="screen">
  <div class="header">
    <div class="header-left">
      <span class="header-logo">DeviceDeck</span>
      <div class="header-status" id="header-status">
        <span class="status-dot" id="status-dot"></span>
        <span id="status-text">已连接</span>
      </div>
    </div>
    <div class="header-actions">
      <button class="icon-btn" id="theme-btn" title="切换主题">
        <svg class="theme-icon theme-icon-sun" xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="4"/><path d="M12 2v2"/><path d="M12 20v2"/><path d="m4.93 4.93 1.41 1.41"/><path d="m17.66 17.66 1.41 1.41"/><path d="M2 12h2"/><path d="M20 12h2"/><path d="m6.34 17.66-1.41 1.41"/><path d="m19.07 4.93-1.41 1.41"/></svg>
        <svg class="theme-icon theme-icon-moon" xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20.985 12.486a9 9 0 1 1-9.473-9.472c.405-.022.617.46.402.803a6 6 0 0 0 8.268 8.268c.344-.215.825-.004.803.401"/></svg>
      </button>
      <button class="icon-btn" id="refresh-btn" title="刷新文件列表">
        <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="23 4 23 10 17 10"/><polyline points="1 20 1 14 7 14"/><path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"/></svg>
      </button>
      <button class="icon-btn" id="disconnect-btn" title="断开连接">
        <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"/><polyline points="16 17 21 12 16 7"/><line x1="21" y1="12" x2="9" y2="12"/></svg>
      </button>
    </div>
  </div>

  <div class="offline-bar" id="offline-bar">连接已断开，正在尝试重新连接…</div>

  <div class="content">
    <div class="upload-zone" id="upload-zone">
      <svg xmlns="http://www.w3.org/2000/svg" width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M4 14.899A7 7 0 1 1 15.71 8h1.79a4.5 4.5 0 0 1 2.5 8.242"/><path d="M12 12v9"/><path d="m8 16 4-4 4 4"/></svg>
      <div class="upload-zone-title">拖拽文件到此处</div>
      <div class="upload-zone-sub">或点击下方按钮选择</div>
      <div class="upload-actions">
        <button class="upload-action-btn" id="pick-files-btn" type="button">
          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7Z"/><path d="M14 2v4a2 2 0 0 0 2 2h4"/></svg>
          选择文件
        </button>
        <button class="upload-action-btn" id="pick-folder-btn" type="button">
          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z"/></svg>
          选择文件夹
        </button>
      </div>
      <input type="file" id="file-input" multiple style="display:none">
      <input type="file" id="folder-input" webkitdirectory multiple style="display:none">
    </div>

    <div class="queue-section" id="queue-section" style="display:none">
      <div class="queue-header">
        <div class="queue-title">
          上传队列
          <span class="count" id="queue-count">0</span>
        </div>
        <div class="queue-header-right">
          <span class="queue-progress-summary" id="queue-summary"></span>
          <button class="queue-clear" id="queue-clear">清空已完成</button>
        </div>
      </div>
      <div class="queue-list" id="queue-list"></div>
      <div class="queue-footer">
        <button class="upload-all-btn" id="upload-all-btn">
          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M5 12h14"/><path d="M12 5v14"/></svg>
          全部上传
        </button>
      </div>
    </div>

    <div class="files-section">
      <div class="files-header">
        <div class="files-title">
          已接收文件
          <span class="count" id="files-count">0</span>
        </div>
        <div class="files-header-right">
          <button class="files-clear-all" id="files-clear-all" style="display:none">
            <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>
            清空全部
          </button>
          <button class="files-download-all" id="files-download-all" style="display:none">
            <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>
            全部下载
          </button>
          <button class="files-refresh" id="files-refresh" title="刷新">
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="23 4 23 10 17 10"/><polyline points="1 20 1 14 7 14"/><path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"/></svg>
          </button>
        </div>
      </div>
      <div id="files-list-wrap">
        <div class="files-tools">
          <input class="files-search" id="files-search" type="search" placeholder="搜索文件">
          <select class="files-select" id="files-type-filter">
            <option value="all">全部类型</option>
            <option value="img">图片</option>
            <option value="video">视频</option>
            <option value="audio">音频</option>
            <option value="doc">文档</option>
            <option value="archive">压缩包</option>
            <option value="code">代码</option>
          </select>
          <div class="dd-lite" data-dropdown-for="files-type-filter"></div>
          <select class="files-select" id="files-sort">
            <option value="name-asc">名称 A-Z</option>
            <option value="name-desc">名称 Z-A</option>
            <option value="size-desc">大小降序</option>
            <option value="size-asc">大小升序</option>
            <option value="time-desc">最新优先</option>
            <option value="time-asc">最早优先</option>
          </select>
          <div class="dd-lite" data-dropdown-for="files-sort"></div>
        </div>
        <div class="files-bulk" id="files-bulk">
          <span id="files-bulk-count">已选择 0 个</span>
          <div class="files-bulk-actions">
            <button class="mini-action" id="files-select-all" type="button">全选</button>
            <button class="mini-action" id="files-bulk-download" type="button">下载</button>
            <button class="mini-action danger" id="files-bulk-delete" type="button">删除</button>
          </div>
        </div>
        <div class="files-list" id="files-list"></div>
        <div class="empty-state" id="files-empty">
          <svg xmlns="http://www.w3.org/2000/svg" width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="22 12 16 12 14 15 10 15 8 12 2 12"/><path d="M5.45 5.11 2 12v6a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-6l-3.45-6.89A2 2 0 0 0 16.76 4H7.24a2 2 0 0 0-1.79 1.11z"/></svg>
          <p>暂无文件</p>
        </div>
      </div>
    </div>

    <div class="history-section">
      <div class="history-header">
        <div class="history-title">传输历史</div>
        <button class="files-refresh" id="history-refresh" title="刷新">
          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="23 4 23 10 17 10"/><polyline points="1 20 1 14 7 14"/><path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"/></svg>
        </button>
      </div>
      <div class="history-list" id="history-list"></div>
    </div>
  </div>
</div>

<div class="modal-overlay" id="modal-overlay">
  <div class="modal">
    <div class="modal-title">断开连接</div>
    <div class="modal-text">确定要断开与电脑的连接吗？正在传输的文件将被取消。</div>
    <div class="modal-actions">
      <button class="modal-btn cancel" id="modal-cancel">取消</button>
      <button class="modal-btn danger" id="modal-confirm">断开</button>
    </div>
  </div>
</div>

<div class="preview-overlay" id="preview-overlay">
  <div class="preview-panel" role="dialog" aria-modal="true" aria-labelledby="preview-title">
    <div class="preview-header">
      <div class="preview-info">
        <div class="preview-name" id="preview-title">预览</div>
        <div class="preview-meta" id="preview-meta"></div>
      </div>
      <div class="preview-actions">
        <button class="icon-btn" id="preview-download" title="下载">
          <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>
        </button>
        <button class="icon-btn" id="preview-close" title="关闭预览">
          <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg>
        </button>
      </div>
    </div>
    <div class="preview-body" id="preview-body"></div>
  </div>
</div>

<div id="toast" class="toast"></div>

<script>
(function() {
  'use strict';

  const $ = s => document.querySelector(s);
  const MAX_FILE_SIZE = 2147483648;
  const TEXT_PREVIEW_LIMIT = 2097152;
  const CHUNK_SIZE = 1024 * 1024;

  let token = '';
  let queue = [];
  let isUploading = false;
  let uploadRequested = false;
  let isConnected = true;
  let heartbeatTimer = null;
  let reconnectTimer = null;
  let eventsSocket = null;
  let eventsReconnectTimer = null;
  let currentPreview = null;
  let previewRequestId = 0;
  const selectedFilePaths = new Set();

  const authScreen = $('#auth');
  const mainScreen = $('#main');
  const tokenInput = $('#token-input');
  const verifyBtn = $('#verify-btn');
  const authError = $('#auth-error');
  const uploadZone = $('#upload-zone');
  const fileInput = $('#file-input');
  const folderInput = $('#folder-input');
  const pickFilesBtn = $('#pick-files-btn');
  const pickFolderBtn = $('#pick-folder-btn');
  const queueSection = $('#queue-section');
  const queueList = $('#queue-list');
  const queueCount = $('#queue-count');
  const queueSummary = $('#queue-summary');
  const queueClear = $('#queue-clear');
  const uploadAllBtn = $('#upload-all-btn');
  const filesList = $('#files-list');
  const filesEmpty = $('#files-empty');
  const filesCount = $('#files-count');
  const filesRefresh = $('#files-refresh');
  const filesDownloadAll = $('#files-download-all');
  const filesClearAll = $('#files-clear-all');
  const filesSearch = $('#files-search');
  const filesTypeFilter = $('#files-type-filter');
  const filesSort = $('#files-sort');
  const filesBulk = $('#files-bulk');
  const filesBulkCount = $('#files-bulk-count');
  const filesSelectAll = $('#files-select-all');
  const filesBulkDownload = $('#files-bulk-download');
  const filesBulkDelete = $('#files-bulk-delete');
  const historyList = $('#history-list');
  const historyRefresh = $('#history-refresh');
  const refreshBtn = $('#refresh-btn');
  const disconnectBtn = $('#disconnect-btn');
  const toastEl = $('#toast');
  const statusDot = $('#status-dot');
  const statusText = $('#status-text');
  const offlineBar = $('#offline-bar');
  const modalOverlay = $('#modal-overlay');
  const modalCancel = $('#modal-cancel');
  const modalConfirm = $('#modal-confirm');
  const themeBtn = $('#theme-btn');
  const authThemeBtn = $('#auth-theme-btn');
  const previewOverlay = $('#preview-overlay');
  const previewTitle = $('#preview-title');
  const previewMeta = $('#preview-meta');
  const previewBody = $('#preview-body');
  const previewClose = $('#preview-close');
  const previewDownload = $('#preview-download');

  const initialLocale = '__DD_LOCALE__';
  const locale = initialLocale === 'en' ? 'en' : 'zh-CN';
  const clientId = getClientId();
  const messages = {
    'zh-CN': {
      pageTitle: 'DeviceDeck 文件传输',
      authSub: 'WiFi 文件传输',
      authLabel: '输入电脑端显示的 8 位连接码',
      tokenPlaceholder: '输入连接码',
      connect: '连接',
      authHint: '在电脑上打开 DeviceDeck 并启动<br>WiFi 传输以获取连接码',
      connected: '已连接',
      disconnected: '已断开',
      reconnecting: '重连中…',
      refreshTitle: '刷新文件列表',
      disconnectTitle: '断开连接',
      themeTitle: '切换主题',
      offlineText: '连接已断开，正在尝试重新连接…',
      uploadTitle: '拖拽文件到此处',
      uploadSub: '或点击下方按钮选择',
      pickFiles: '选择文件',
      pickFolder: '选择文件夹',
      queueTitle: '上传队列',
      clearDone: '清空已完成',
      uploadAll: '全部上传',
      uploading: '上传中…',
      receivedFiles: '已接收文件',
      clearAll: '清空全部',
      downloadAll: '全部下载',
      refresh: '刷新',
      noFiles: '暂无文件',
      modalTitle: '断开连接',
      modalText: '确定要断开与电脑的连接吗？正在传输的文件将被取消。',
      cancel: '取消',
      disconnect: '断开',
      invalidToken: '连接码无效，请检查后重试',
      serverUnavailable: '无法连接到服务器',
      reconnected: '已重新连接',
      duplicateSkipped: ' 个重复文件已跳过',
      oversizedSkipped: ' 个文件超过 2GB 大小限制',
      completed: '已完成',
      failed: '失败',
      waiting: '等待中',
      paused: '已暂停',
      pause: '暂停',
      resume: '继续',
      retry: '重试',
      remove: '移除',
      cancelled: '已取消: ',
      uploadComplete: ' 个文件上传完成',
      successCount: ' 个成功，',
      failCount: ' 个失败',
      uploadFailed: ' 个文件上传失败',
      delete: '删除',
      download: '下载',
      deleted: '已删除: ',
      deleteFailed: '删除失败',
      deletedCountPrefix: '已删除 ',
      deletedCountSuffix: ' 个文件',
      downloadStart: '开始下载 ',
      fileCountSuffix: ' 个文件',
      folder: '文件夹',
      loading: '加载中…',
      toggleFolder: '展开/收起文件夹',
      folderDropUnsupported: '当前浏览器不支持拖拽目录，请使用“选择文件夹”',
      folderPickerUnsupported: '当前浏览器不支持选择文件夹，请选择多个文件或在桌面端使用',
      folderPathUnavailable: '当前浏览器没有提供文件夹路径，已按普通文件加入队列',
      preview: '预览',
      closePreview: '关闭预览',
      previewLoading: '正在加载预览…',
      previewFailed: '预览加载失败',
      unsupportedPreview: '此文件类型暂不支持预览',
      fileTooLargePreview: '文本文件超过 2MB，无法在浏览器内预览',
      searchFiles: '搜索文件',
      allTypes: '全部类型',
      typeImage: '图片',
      typeVideo: '视频',
      typeAudio: '音频',
      typeDoc: '文档',
      typeArchive: '压缩包',
      typeCode: '代码',
      sortNameAsc: '名称 A-Z',
      sortNameDesc: '名称 Z-A',
      sortSizeDesc: '大小降序',
      sortSizeAsc: '大小升序',
      sortTimeDesc: '最新优先',
      sortTimeAsc: '最早优先',
      selectedCount: '已选择 ',
      selectedSuffix: ' 个',
      selectAll: '全选',
      history: '传输历史',
      eta: '剩余 ',
      verified: '已校验',
      folderStructureHint: '支持时会保留文件夹结构；不支持时按普通文件上传。'
    },
    en: {
      pageTitle: 'DeviceDeck File Transfer',
      authSub: 'WiFi File Transfer',
      authLabel: 'Enter the 8-digit code shown on your computer',
      tokenPlaceholder: 'Enter code',
      connect: 'Connect',
      authHint: 'Open DeviceDeck on your computer and start<br>WiFi Transfer to get the code',
      connected: 'Connected',
      disconnected: 'Disconnected',
      reconnecting: 'Reconnecting…',
      refreshTitle: 'Refresh file list',
      disconnectTitle: 'Disconnect',
      themeTitle: 'Toggle theme',
      offlineText: 'Connection lost. Trying to reconnect…',
      uploadTitle: 'Drop files here',
      uploadSub: 'Or choose files below',
      pickFiles: 'Choose files',
      pickFolder: 'Choose folder',
      queueTitle: 'Upload queue',
      clearDone: 'Clear completed',
      uploadAll: 'Upload all',
      uploading: 'Uploading…',
      receivedFiles: 'Received files',
      clearAll: 'Clear all',
      downloadAll: 'Download all',
      refresh: 'Refresh',
      noFiles: 'No files',
      modalTitle: 'Disconnect',
      modalText: 'Disconnect from this computer? Active transfers will be cancelled.',
      cancel: 'Cancel',
      disconnect: 'Disconnect',
      invalidToken: 'Invalid code. Check it and try again.',
      serverUnavailable: 'Cannot connect to the server',
      reconnected: 'Reconnected',
      duplicateSkipped: ' duplicate files skipped',
      oversizedSkipped: ' files exceed the 2GB limit',
      completed: 'completed',
      failed: 'failed',
      waiting: 'Waiting',
      paused: 'Paused',
      pause: 'Pause',
      resume: 'Resume',
      retry: 'Retry',
      remove: 'Remove',
      cancelled: 'Cancelled: ',
      uploadComplete: ' files uploaded',
      successCount: ' succeeded, ',
      failCount: ' failed',
      uploadFailed: ' files failed to upload',
      delete: 'Delete',
      download: 'Download',
      deleted: 'Deleted: ',
      deleteFailed: 'Delete failed',
      deletedCountPrefix: 'Deleted ',
      deletedCountSuffix: ' files',
      downloadStart: 'Downloading ',
      fileCountSuffix: ' files',
      folder: 'Folder',
      loading: 'Loading…',
      toggleFolder: 'Expand or collapse folder',
      folderDropUnsupported: 'This browser does not support folder drag-and-drop. Use Choose folder instead.',
      folderPickerUnsupported: 'This browser cannot choose folders. Select multiple files or use desktop web.',
      folderPathUnavailable: 'This browser did not provide folder paths. Files were queued flat.',
      preview: 'Preview',
      closePreview: 'Close preview',
      previewLoading: 'Loading preview…',
      previewFailed: 'Failed to load preview',
      unsupportedPreview: 'Preview is not available for this file type',
      fileTooLargePreview: 'Text files over 2MB cannot be previewed in the browser',
      searchFiles: 'Search files',
      allTypes: 'All types',
      typeImage: 'Images',
      typeVideo: 'Videos',
      typeAudio: 'Audio',
      typeDoc: 'Documents',
      typeArchive: 'Archives',
      typeCode: 'Code',
      sortNameAsc: 'Name A-Z',
      sortNameDesc: 'Name Z-A',
      sortSizeDesc: 'Size high-low',
      sortSizeAsc: 'Size low-high',
      sortTimeDesc: 'Newest first',
      sortTimeAsc: 'Oldest first',
      selectedCount: 'Selected ',
      selectedSuffix: '',
      selectAll: 'Select all',
      history: 'Transfer history',
      eta: 'ETA ',
      verified: 'Verified',
      folderStructureHint: 'Folder structure is preserved when the browser provides paths.'
    }
  };
  const text = messages[locale];

  function t(key) {
    return text[key] || messages['zh-CN'][key] || key;
  }

  function setText(selector, key) {
    const el = $(selector);
    if (el) el.textContent = t(key);
  }

  function setHtml(selector, key) {
    const el = $(selector);
    if (el) el.innerHTML = t(key);
  }

  function setTitle(el, key) {
    if (el) el.title = t(key);
  }

  function setOptionLabel(select, value, key) {
    const option = Array.from(select.options).find(option => option.value === value);
    if (option) option.textContent = t(key);
  }

  function applyLocale() {
    document.documentElement.lang = locale;
    document.title = t('pageTitle');
    setText('.auth-sub', 'authSub');
    setText('.auth-label', 'authLabel');
    tokenInput.placeholder = t('tokenPlaceholder');
    verifyBtn.lastChild.textContent = t('connect');
    setHtml('.auth-hint', 'authHint');
    statusText.textContent = t('connected');
    setTitle(refreshBtn, 'refreshTitle');
    setTitle(disconnectBtn, 'disconnectTitle');
    setTitle(themeBtn, 'themeTitle');
    setTitle(authThemeBtn, 'themeTitle');
    setTitle(pickFolderBtn, supportsFolderPicker() ? 'pickFolder' : 'folderPickerUnsupported');
    offlineBar.textContent = t('offlineText');
    setText('.upload-zone-title', 'uploadTitle');
    setText('.upload-zone-sub', 'uploadSub');
    pickFilesBtn.lastChild.textContent = t('pickFiles');
    pickFolderBtn.lastChild.textContent = t('pickFolder');
    const queueTitleText = $('.queue-title')?.firstChild;
    if (queueTitleText) queueTitleText.nodeValue = t('queueTitle') + ' ';
    queueCount.textContent = '0';
    queueClear.textContent = t('clearDone');
    uploadAllBtn.lastChild.textContent = t('uploadAll');
    const filesTitleText = $('.files-title')?.firstChild;
    if (filesTitleText) filesTitleText.nodeValue = t('receivedFiles') + ' ';
    filesCount.textContent = '0';
    filesClearAll.lastChild.textContent = t('clearAll');
    filesDownloadAll.lastChild.textContent = t('downloadAll');
    filesSearch.placeholder = t('searchFiles');
    setOptionLabel(filesTypeFilter, 'all', 'allTypes');
    setOptionLabel(filesTypeFilter, 'img', 'typeImage');
    setOptionLabel(filesTypeFilter, 'video', 'typeVideo');
    setOptionLabel(filesTypeFilter, 'audio', 'typeAudio');
    setOptionLabel(filesTypeFilter, 'doc', 'typeDoc');
    setOptionLabel(filesTypeFilter, 'archive', 'typeArchive');
    setOptionLabel(filesTypeFilter, 'code', 'typeCode');
    setOptionLabel(filesSort, 'name-asc', 'sortNameAsc');
    setOptionLabel(filesSort, 'name-desc', 'sortNameDesc');
    setOptionLabel(filesSort, 'size-desc', 'sortSizeDesc');
    setOptionLabel(filesSort, 'size-asc', 'sortSizeAsc');
    setOptionLabel(filesSort, 'time-desc', 'sortTimeDesc');
    setOptionLabel(filesSort, 'time-asc', 'sortTimeAsc');
    filesSelectAll.textContent = t('selectAll');
    filesBulkDownload.textContent = t('download');
    filesBulkDelete.textContent = t('delete');
    setText('.history-title', 'history');
    setTitle(filesRefresh, 'refresh');
    setTitle(historyRefresh, 'refresh');
    setText('#files-empty p', 'noFiles');
    setText('.modal-title', 'modalTitle');
    setText('.modal-text', 'modalText');
    modalCancel.textContent = t('cancel');
    modalConfirm.textContent = t('disconnect');
    previewTitle.textContent = t('preview');
    setTitle(previewClose, 'closePreview');
    setTitle(previewDownload, 'download');
  }

  function setTheme(theme) {
    const next = theme === 'light' ? 'light' : 'dark';
    document.documentElement.dataset.theme = next;
    try { localStorage.setItem('devicedeck-wifi-theme', next); } catch {}
  }

  function toggleTheme() {
    setTheme(document.documentElement.dataset.theme === 'light' ? 'dark' : 'light');
  }

  function initCustomDropdowns() {
    document.querySelectorAll('[data-dropdown-for]').forEach(root => {
      const select = document.getElementById(root.dataset.dropdownFor);
      if (!select) return;
      root.tabIndex = 0;
      renderCustomDropdown(root, select, false, -1);
      root.addEventListener('click', e => {
        e.stopPropagation();
        const option = e.target.closest('[data-dd-value]');
        if (option) {
          select.value = option.dataset.ddValue;
          select.dispatchEvent(new Event('change', { bubbles: true }));
          renderCustomDropdown(root, select, false, -1);
          return;
        }
        renderCustomDropdown(root, select, !root.classList.contains('open'), -1);
      });
      root.addEventListener('keydown', e => handleDropdownKey(root, select, e));
    });
    document.addEventListener('click', e => {
      document.querySelectorAll('.dd-lite.open').forEach(root => {
        if (!root.contains(e.target)) {
          const select = document.getElementById(root.dataset.dropdownFor);
          if (select) renderCustomDropdown(root, select, false, -1);
        }
      });
    });
    window.addEventListener('resize', closeOpenDropdowns);
    window.addEventListener('scroll', closeOpenDropdowns, true);
  }

  function closeOpenDropdowns() {
    document.querySelectorAll('.dd-lite.open').forEach(root => {
      const select = document.getElementById(root.dataset.dropdownFor);
      if (select) renderCustomDropdown(root, select, false, -1);
    });
  }

  function getDropdownPanelStyle(root) {
    const rect = root.getBoundingClientRect();
    const gap = 4;
    const margin = 12;
    const width = Math.min(rect.width, window.innerWidth - margin * 2);
    const left = Math.max(margin, Math.min(rect.left, window.innerWidth - width - margin));
    const top = rect.bottom + gap;
    const availableHeight = Math.max(32, window.innerHeight - top - margin);
    return [
      'left:' + left + 'px',
      'top:' + top + 'px',
      'width:' + width + 'px',
      'max-height:' + Math.min(220, availableHeight) + 'px'
    ].join(';');
  }

  function renderCustomDropdown(root, select, open, highlightIndex) {
    const options = Array.from(select.options);
    const selectedIndex = Math.max(0, options.findIndex(option => option.value === select.value));
    const activeIndex = highlightIndex >= 0 ? highlightIndex : selectedIndex;
    const selected = options[selectedIndex] || options[0];
    root.classList.toggle('open', open);
    root.innerHTML =
      '<button class="dd-lite-trigger" type="button" aria-haspopup="listbox" aria-expanded="' + String(open) + '">' +
        '<span class="dd-lite-label">' + esc(selected ? selected.textContent : '') + '</span>' +
        '<span class="dd-lite-chevron"><svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m6 9 6 6 6-6"/></svg></span>' +
      '</button>' +
      (open
        ? '<div class="dd-lite-panel" role="listbox" style="' + getDropdownPanelStyle(root) + '">' + options.map((option, index) =>
            '<div class="dd-lite-option' +
              (option.value === select.value ? ' active' : '') +
              (index === activeIndex ? ' highlight' : '') +
              '" role="option" aria-selected="' + String(option.value === select.value) + '" data-dd-value="' + esc(option.value) + '">' +
              '<span>' + esc(option.textContent) + '</span>' +
              (option.value === select.value ? '<span class="dd-lite-check">✓</span>' : '') +
            '</div>'
          ).join('') + '</div>'
        : '');
    root.dataset.highlightIndex = String(activeIndex);
  }

  function handleDropdownKey(root, select, e) {
    const options = Array.from(select.options);
    const open = root.classList.contains('open');
    const current = Number(root.dataset.highlightIndex || options.findIndex(option => option.value === select.value) || 0);
    if (!open && ['Enter',' ','ArrowDown','ArrowUp'].includes(e.key)) {
      e.preventDefault();
      renderCustomDropdown(root, select, true, current);
      return;
    }
    if (!open) return;
    if (e.key === 'Escape') {
      e.preventDefault();
      renderCustomDropdown(root, select, false, -1);
      return;
    }
    if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
      e.preventDefault();
      const next = e.key === 'ArrowDown'
        ? (current + 1) % options.length
        : (current - 1 + options.length) % options.length;
      renderCustomDropdown(root, select, true, next);
      return;
    }
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      const option = options[current];
      if (option) {
        select.value = option.value;
        select.dispatchEvent(new Event('change', { bubbles: true }));
      }
      renderCustomDropdown(root, select, false, -1);
    }
  }

  applyLocale();
  updateFolderPickerAvailability();
  initCustomDropdowns();
  themeBtn.addEventListener('click', toggleTheme);
  authThemeBtn.addEventListener('click', toggleTheme);

  const urlToken = new URLSearchParams(location.search).get('token') || '';
  if (urlToken) {
    tokenInput.value = urlToken;
    verify();
  }

  tokenInput.addEventListener('input', () => {
    verifyBtn.disabled = tokenInput.value.trim().length < 1;
    authError.textContent = '';
  });
  tokenInput.addEventListener('keydown', e => {
    if (e.key === 'Enter' && !verifyBtn.disabled) verify();
  });
  verifyBtn.addEventListener('click', verify);

  async function verify() {
    const code = tokenInput.value.trim();
    if (!code) return;
    verifyBtn.disabled = true;
    authError.textContent = '';
    try {
      const r = await fetch('/api/verify?token=' + encodeURIComponent(code));
      const d = await r.json();
      if (d.valid) {
        token = code;
        showScreen('main');
        startHeartbeat();
        connectEventSocket();
        loadFiles();
      } else {
        authError.textContent = t('invalidToken');
        authError.classList.remove('shake');
        void authError.offsetWidth;
        authError.classList.add('shake');
      }
    } catch {
      authError.textContent = t('serverUnavailable');
      authError.classList.remove("shake");
      void authError.offsetWidth;
      authError.classList.add("shake");
    }
    verifyBtn.disabled = false;
  }

  function showScreen(name) {
    authScreen.classList.remove('active');
    mainScreen.classList.remove('active');
    (name === 'auth' ? authScreen : mainScreen).classList.add('active');
    if (name === 'main') mainScreen.classList.add('fade-in');
  }

  disconnectBtn.addEventListener('click', () => {
    const hasUploading = queue.some(q => q.status === 'uploading');
    if (hasUploading) {
      modalOverlay.classList.add('show');
    } else {
      doDisconnect();
    }
  });
  modalCancel.addEventListener('click', () => modalOverlay.classList.remove('show'));
  modalConfirm.addEventListener('click', () => {
    modalOverlay.classList.remove('show');
    doDisconnect();
  });
  modalOverlay.addEventListener('click', e => {
    if (e.target === modalOverlay) modalOverlay.classList.remove('show');
  });

  function doDisconnect() {
    closePreview();
    queue.forEach(q => {
      if (q.xhr) { try { q.xhr.abort(); } catch {} }
    });
    token = '';
    queue = [];
    isUploading = false;
    uploadRequested = false;
    queueSection.style.display = 'none';
    tokenInput.value = '';
    stopHeartbeat();
    setConnectionState(true);
    showScreen('auth');
  }

  function startHeartbeat() {
    stopHeartbeat();
    heartbeatTimer = setInterval(checkConnection, 5000);
  }

  function stopHeartbeat() {
    if (heartbeatTimer) { clearInterval(heartbeatTimer); heartbeatTimer = null; }
    if (reconnectTimer) { clearTimeout(reconnectTimer); reconnectTimer = null; }
    closeEventSocket();
  }

  async function checkConnection() {
    try {
      const r = await fetch('/api/verify?token=' + encodeURIComponent(token), { signal: AbortSignal.timeout(3000) });
      const d = await r.json();
      if (d.valid) {
        if (!isConnected) setConnectionState(true);
      } else {
        if (isConnected) setConnectionState(false);
      }
    } catch {
      if (isConnected) setConnectionState(false);
    }
  }

  function setConnectionState(connected) {
    isConnected = connected;
    if (connected) {
      statusDot.className = 'status-dot';
      statusText.textContent = t('connected');
      offlineBar.className = 'offline-bar';
      uploadZone.classList.remove('disabled');
      if (reconnectTimer) { clearTimeout(reconnectTimer); reconnectTimer = null; }
    } else {
      statusDot.className = 'status-dot disconnected';
      statusText.textContent = t('disconnected');
      offlineBar.className = 'offline-bar show';
      uploadZone.classList.add('disabled');
      scheduleReconnect();
    }
  }

  function scheduleReconnect() {
    if (reconnectTimer) return;
    statusDot.className = 'status-dot reconnecting';
    statusText.textContent = t('reconnecting');
    offlineBar.className = 'offline-bar show reconnecting';
    reconnectTimer = setTimeout(async () => {
      reconnectTimer = null;
      try {
        const r = await fetch('/api/verify?token=' + encodeURIComponent(token), { signal: AbortSignal.timeout(3000) });
        const d = await r.json();
        if (d.valid) {
          setConnectionState(true);
          connectEventSocket();
          showToast('success', t('reconnected'));
          loadFiles();
        } else {
          scheduleReconnect();
        }
      } catch {
        scheduleReconnect();
      }
    }, 3000);
  }

  function getClientId() {
    try {
      const key = 'devicedeck-wifi-client-id';
      let value = localStorage.getItem(key);
      if (!value) {
        value = Math.random().toString(36).slice(2) + Date.now().toString(36);
        localStorage.setItem(key, value);
      }
      return value;
    } catch {
      return Math.random().toString(36).slice(2) + Date.now().toString(36);
    }
  }

  function eventSocketUrl() {
    const protocol = location.protocol === 'https:' ? 'wss:' : 'ws:';
    const params = new URLSearchParams({ token, clientId });
    return protocol + '//' + location.host + '/ws?' + params.toString();
  }

  function connectEventSocket() {
    if (!token || (eventsSocket && eventsSocket.readyState <= WebSocket.OPEN)) return;
    if (eventsReconnectTimer) {
      clearTimeout(eventsReconnectTimer);
      eventsReconnectTimer = null;
    }

    const socket = new WebSocket(eventSocketUrl());
    eventsSocket = socket;

    socket.addEventListener('open', () => {
      if (eventsSocket !== socket) return;
      if (!isConnected) setConnectionState(true);
    });
    socket.addEventListener('message', e => {
      if (eventsSocket !== socket) return;
      try {
        const event = JSON.parse(e.data);
        if (event.actorClientId && event.actorClientId === clientId) return;
        loadFiles();
      } catch {}
    });
    socket.addEventListener('close', () => {
      if (eventsSocket !== socket) return;
      eventsSocket = null;
      scheduleEventSocketReconnect();
    });
    socket.addEventListener('error', () => {
      try { socket.close(); } catch {}
    });
  }

  function closeEventSocket() {
    if (eventsReconnectTimer) {
      clearTimeout(eventsReconnectTimer);
      eventsReconnectTimer = null;
    }
    if (eventsSocket) {
      const socket = eventsSocket;
      eventsSocket = null;
      try { socket.close(); } catch {}
    }
  }

  function scheduleEventSocketReconnect() {
    if (!token || eventsReconnectTimer) return;
    eventsReconnectTimer = setTimeout(() => {
      eventsReconnectTimer = null;
      connectEventSocket();
    }, 3000);
  }

  async function refreshAfterResume() {
    if (!token || !mainScreen.classList.contains('active')) return;
    await checkConnection();
    if (isConnected) loadFiles();
  }

  document.addEventListener('visibilitychange', () => {
    if (!document.hidden) refreshAfterResume();
  });
  window.addEventListener('pageshow', refreshAfterResume);

  fileInput.addEventListener('click', e => e.stopPropagation());
  folderInput.addEventListener('click', e => e.stopPropagation());
  pickFilesBtn.addEventListener('click', e => {
    e.preventDefault();
    e.stopPropagation();
    fileInput.click();
  });
  pickFolderBtn.addEventListener('click', e => {
    e.preventDefault();
    e.stopPropagation();
    if (!supportsFolderPicker()) {
      showToast('warning', t('folderPickerUnsupported'));
      return;
    }
    showToast('info', t('folderStructureHint'));
    folderInput.click();
  });
  uploadZone.addEventListener('click', e => {
    if (e.target.closest('.upload-actions') || e.target === fileInput || e.target === folderInput) return;
    fileInput.click();
  });

  fileInput.addEventListener('change', e => { addToQueue(e.target.files); fileInput.value = ''; });
  folderInput.addEventListener('change', e => handleFolderInputChange(e));

  function supportsFolderPicker() {
    const input = document.createElement('input');
    input.type = 'file';
    return 'webkitdirectory' in input;
  }

  function updateFolderPickerAvailability() {
    const supported = supportsFolderPicker();
    pickFolderBtn.classList.toggle('unsupported', !supported);
    pickFolderBtn.setAttribute('aria-disabled', String(!supported));
    pickFolderBtn.title = supported ? t('pickFolder') : t('folderPickerUnsupported');
  }

  function handleFolderInputChange(event) {
    const files = Array.from(event.target.files || []);
    if (files.length > 0 && !files.some(file => file.webkitRelativePath)) {
      showToast('warning', t('folderPathUnavailable'));
    }
    addToQueue(files);
    folderInput.value = '';
  }

  uploadZone.addEventListener('dragover', e => { e.preventDefault(); uploadZone.classList.add('dragover'); });
  uploadZone.addEventListener('dragleave', () => uploadZone.classList.remove('dragover'));
  uploadZone.addEventListener('drop', async e => {
    e.preventDefault();
    uploadZone.classList.remove('dragover');
    const files = await collectDroppedFiles(e.dataTransfer);
    if (files.length > 0) {
      addToQueue(files);
    } else if (e.dataTransfer.files.length > 0) {
      addToQueue(e.dataTransfer.files);
    }
  });

  function queueFilePath(file) {
    return file.webkitRelativePath || file.relativePath || file.name;
  }

  const expandedQueueGroups = new Set();

  function queueGroupKey(item) {
    const path = item.path || item.file.name;
    const parts = path.split('/').filter(Boolean);
    return parts.length > 1 ? parts[0] : '';
  }

  function queueGroupStats(items) {
    const totalBytes = items.reduce((sum, item) => sum + item.file.size, 0);
    const done = items.filter(item => item.status === 'done').length;
    const failed = items.filter(item => item.status === 'error').length;
    const uploading = items.find(item => item.status === 'uploading');
    const uploadedBytes = items.reduce((sum, item) => {
      if (item.status === 'done') return sum + item.file.size;
      if (item.status === 'uploading') return sum + Math.round(item.file.size * item.progress / 100);
      return sum;
    }, 0);
    return {
      totalBytes,
      done,
      failed,
      uploading,
      progress: totalBytes > 0 ? Math.round(uploadedBytes / totalBytes * 100) : 0
    };
  }

  function addToQueue(fileList) {
    let added = 0, skipped = 0, oversized = 0;
    for (const f of fileList) {
      const filePath = queueFilePath(f);
      const dup = queue.some(q => q.path === filePath && q.file.size === f.size && q.status !== 'done');
      if (dup) { skipped++; continue; }
      if (f.size > MAX_FILE_SIZE) { oversized++; continue; }
      queue.push({ id: Math.random().toString(36).slice(2), file: f, path: filePath, status: 'waiting', progress: 0, xhr: null });
      added++;
    }
    if (skipped > 0) showToast('warning', skipped + t('duplicateSkipped'));
    if (oversized > 0) showToast('error', oversized + t('oversizedSkipped'));
    if (added > 0) renderQueue();
  }

  async function collectDroppedFiles(dataTransfer) {
    const items = Array.from(dataTransfer.items || []);
    if (items.length === 0) return Array.from(dataTransfer.files || []);

    const entries = items
      .map(item => item.webkitGetAsEntry ? item.webkitGetAsEntry() : null)
      .filter(Boolean);
    if (entries.length === 0) return Array.from(dataTransfer.files || []);

    try {
      const groups = await Promise.all(entries.map(entry => readEntryFiles(entry, '')));
      return groups.flat();
    } catch {
      showToast('warning', t('folderDropUnsupported'));
      return Array.from(dataTransfer.files || []);
    }
  }

  function readEntryFiles(entry, parentPath) {
    const entryPath = parentPath ? parentPath + '/' + entry.name : entry.name;
    if (entry.isFile) {
      return new Promise((resolve, reject) => {
        entry.file(file => {
          try { Object.defineProperty(file, 'relativePath', { value: entryPath }); } catch {}
          resolve([file]);
        }, reject);
      });
    }

    if (entry.isDirectory) {
      const reader = entry.createReader();
      const allEntries = [];
      return new Promise((resolve, reject) => {
        const readBatch = () => {
          reader.readEntries(async batch => {
            if (batch.length === 0) {
              try {
                const groups = await Promise.all(allEntries.map(child => readEntryFiles(child, entryPath)));
                resolve(groups.flat());
              } catch (err) {
                reject(err);
              }
              return;
            }
            allEntries.push(...batch);
            readBatch();
          }, reject);
        };
        readBatch();
      });
    }

    return Promise.resolve([]);
  }

  function renderQueue() {
    if (queue.length === 0) {
      queueSection.style.display = 'none';
      return;
    }
    queueSection.style.display = '';
    queueCount.textContent = queue.length;

    const done = queue.filter(q => q.status === 'done').length;
    const err = queue.filter(q => q.status === 'error').length;
    const total = queue.length;
    queueSummary.textContent = done + '/' + total + ' ' + t('completed') + (err > 0 ? (locale === 'en' ? ', ' : '，') + err + ' ' + t('failed') : '');

    const hasWaiting = queue.some(q => q.status === 'waiting' || q.status === 'error');
    uploadAllBtn.disabled = !hasWaiting || isUploading;
    uploadAllBtn.innerHTML = (hasWaiting && !isUploading
      ? '<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M5 12h14"/><path d="M12 5v14"/></svg>' + t('uploadAll')
      : '<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg>' + t('uploading'));

    const groups = new Map();
    const singles = [];
    queue.forEach(item => {
      const groupKey = queueGroupKey(item);
      if (!groupKey) {
        singles.push(item);
        return;
      }
      if (!groups.has(groupKey)) groups.set(groupKey, []);
      groups.get(groupKey).push(item);
    });

    queueList.innerHTML = '';

    groups.forEach((items, groupKey) => {
      const el = document.createElement('div');
      el.className = 'queue-folder';
      el.dataset.group = groupKey;
      queueList.appendChild(el);
      updateQueueGroup(el, groupKey, items);
    });

    singles.forEach(item => {
      const el = document.createElement('div');
      el.className = 'queue-item';
      el.dataset.qid = item.id;
      queueList.appendChild(el);
      updateQueueItem(el, item);
    });
  }

  function updateQueueGroup(el, groupKey, items) {
    const expanded = expandedQueueGroups.has(groupKey);
    const stats = queueGroupStats(items);
    const pausedCount = items.filter(item => item.status === 'paused').length;
    const canPause = items.some(item => item.status === 'waiting' || item.status === 'error' || item.status === 'uploading');
    const canResume = pausedCount > 0;
    const canCancel = items.some(item => item.status !== 'done');
    const status = stats.uploading
      ? stats.progress + '%'
      : pausedCount > 0
        ? pausedCount + ' ' + t('paused')
        : stats.failed > 0
        ? stats.failed + ' ' + t('failed')
        : stats.done === items.length
          ? t('completed')
          : t('waiting');
    const statusClass = stats.uploading ? 'uploading' : pausedCount > 0 ? 'paused' : stats.failed > 0 ? 'error' : stats.done === items.length ? 'done' : 'waiting';
    const chevron = expanded
      ? '<path d="m6 9 6 6 6-6"/>'
      : '<path d="m9 18 6-6-6-6"/>';
    const childHtml = expanded
      ? '<div class="queue-folder-children"></div>'
      : '';
    const pauseIcon = '<svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="14" y="4" width="4" height="16" rx="1"/><rect x="6" y="4" width="4" height="16" rx="1"/></svg>';
    const resumeIcon = '<svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><path d="M8 5v14l11-7z"/></svg>';
    const cancelIcon = '<svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg>';
    const actionsHTML = canPause || canResume || canCancel
      ? '<div class="queue-folder-actions">' +
          (canPause ? '<button class="queue-item-action pause" data-action="pause-group" type="button" title="' + t('pause') + '">' + pauseIcon + '</button>' : '') +
          (canResume ? '<button class="queue-item-action resume" data-action="resume-group" type="button" title="' + t('resume') + '">' + resumeIcon + '</button>' : '') +
          (canCancel ? '<button class="queue-item-action" data-action="cancel-group" type="button" title="' + t('cancel') + '">' + cancelIcon + '</button>' : '') +
        '</div>'
      : '';

    el.innerHTML =
      '<div class="queue-folder-head" data-action="toggle-group" data-group="' + esc(groupKey) + '">' +
        '<button class="queue-folder-toggle" type="button" title="' + t('toggleFolder') + '">' +
          '<svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">' + chevron + '</svg>' +
        '</button>' +
        '<div class="queue-item-icon folder">' +
          '<svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z"/></svg>' +
        '</div>' +
        '<div class="queue-item-info">' +
          '<div class="queue-item-name">' + esc(groupKey) + '</div>' +
          '<div class="queue-item-meta"><span>' + items.length + t('fileCountSuffix') + '</span><span>' + formatSize(stats.totalBytes) + '</span><span class="queue-item-status ' + statusClass + '">' + status + '</span></div>' +
          (stats.uploading ? '<div class="queue-progress"><div class="queue-progress-bar" style="width:' + stats.progress + '%"></div></div>' : '') +
        '</div>' +
        actionsHTML +
      '</div>' +
      childHtml;

    el.querySelector('[data-action="toggle-group"]').addEventListener('click', () => {
      if (expandedQueueGroups.has(groupKey)) expandedQueueGroups.delete(groupKey);
      else expandedQueueGroups.add(groupKey);
      renderQueue();
    });
    el.querySelectorAll('.queue-folder-actions [data-action]').forEach(btn => {
      btn.addEventListener('click', e => {
        e.stopPropagation();
        const action = btn.dataset.action;
        if (action === 'pause-group') pauseQueueGroup(groupKey);
        else if (action === 'resume-group') resumeQueueGroup(groupKey);
        else if (action === 'cancel-group') cancelQueueGroup(groupKey);
      });
    });

    if (expanded) {
      const children = el.querySelector('.queue-folder-children');
      items.forEach(item => {
        const child = document.createElement('div');
        child.className = 'queue-item queue-item-child';
        child.dataset.qid = item.id;
        children.appendChild(child);
        updateQueueItem(child, item);
      });
    }
  }

  function updateQueueItem(el, item) {
    const displayName = item.path || item.file.name;
    const type = fileType(item.file.name);
    const statusLabel = {
      uploading: item.progress + '%',
      waiting: t('waiting'),
      paused: t('paused'),
      done: t('completed'),
      error: t('failed')
    }[item.status];

    const iconHTML = '<div class="queue-item-icon">' + fileIconSVG(item.file.name, type) + '</div>';
    const metaHTML = '<span class="queue-speed">' + formatUploadMeta(item) + '</span>' +
      '<span class="queue-item-status ' + item.status + '">' + statusLabel + '</span>';
    const progressHTML = item.status === 'uploading'
      ? '<div class="queue-progress"><div class="queue-progress-bar" style="width:' + item.progress + '%"></div></div>'
      : '';

    const pauseIcon = '<svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="14" y="4" width="4" height="16" rx="1"/><rect x="6" y="4" width="4" height="16" rx="1"/></svg>';
    const resumeIcon = '<svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><path d="M8 5v14l11-7z"/></svg>';
    const cancelIcon = '<svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg>';
    const retryIcon = '<svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="23 4 23 10 17 10"/><polyline points="1 20 1 14 7 14"/><path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"/></svg>';
    let actionHTML;
    if (item.status === 'uploading') {
      actionHTML = '<div class="queue-item-actions">' +
        '<button class="queue-item-action pause" data-action="pause" data-id="' + item.id + '" title="' + t('pause') + '">' + pauseIcon + '</button>' +
        '<button class="queue-item-action" data-action="cancel" data-id="' + item.id + '" title="' + t('cancel') + '">' + cancelIcon + '</button>' +
        '</div>';
    } else if (item.status === 'paused') {
      actionHTML = '<div class="queue-item-actions">' +
        '<button class="queue-item-action resume" data-action="resume" data-id="' + item.id + '" title="' + t('resume') + '">' + resumeIcon + '</button>' +
        '<button class="queue-item-action" data-action="cancel" data-id="' + item.id + '" title="' + t('cancel') + '">' + cancelIcon + '</button>' +
        '</div>';
    } else if (item.status === 'error') {
      actionHTML = '<div class="queue-item-actions">' +
        '<button class="queue-item-action pause" data-action="pause" data-id="' + item.id + '" title="' + t('pause') + '">' + pauseIcon + '</button>' +
        '<button class="queue-item-action retry" data-action="retry" data-id="' + item.id + '" title="' + t('retry') + '">' + retryIcon + '</button>' +
        '<button class="queue-item-action" data-action="cancel" data-id="' + item.id + '" title="' + t('cancel') + '">' + cancelIcon + '</button>' +
        '</div>';
    } else if (item.status === 'done') {
      actionHTML = '';
    } else {
      actionHTML = '<div class="queue-item-actions">' +
        '<button class="queue-item-action pause" data-action="pause" data-id="' + item.id + '" title="' + t('pause') + '">' + pauseIcon + '</button>' +
        '<button class="queue-item-action" data-action="cancel" data-id="' + item.id + '" title="' + t('cancel') + '">' + cancelIcon + '</button>' +
        '</div>';
    }

    el.innerHTML = iconHTML +
      '<div class="queue-item-info">' +
        '<div class="queue-item-name">' + esc(displayName) + '</div>' +
        '<div class="queue-item-meta">' + metaHTML + '</div>' +
        progressHTML +
      '</div>' + actionHTML;

    el.querySelectorAll('[data-action]').forEach(btn => {
      btn.addEventListener('click', e => {
        e.stopPropagation();
        const action = btn.dataset.action;
        const id = btn.dataset.id;
        if (action === 'pause') pauseUpload(id);
        else if (action === 'resume') resumeUpload(id);
        else if (action === 'cancel') cancelUpload(id);
        else if (action === 'retry') retryUpload(id);
        else if (action === 'remove') removeFromQueue(id);
      });
    });
  }

  function removeFromQueue(id) {
    queue = queue.filter(q => q.id !== id);
    const el = queueList.querySelector('[data-qid="' + id + '"]');
    if (el) el.remove();
    if (queue.length === 0) queueSection.style.display = 'none';
    else renderQueue();
  }

  function pauseUpload(id) {
    const item = queue.find(q => q.id === id);
    if (!item || item.status === 'done' || item.status === 'paused') return;
    item.status = 'paused';
    item.progress = 0;
    if (item.xhr) { try { item.xhr.abort(); } catch {} item.xhr = null; }
    renderQueue();
  }

  function resumeUpload(id) {
    const item = queue.find(q => q.id === id);
    if (!item || item.status !== 'paused') return;
    item.status = 'waiting';
    item.progress = 0;
    renderQueue();
    startOrQueueUpload();
  }

  function cancelUpload(id) {
    const item = queue.find(q => q.id === id);
    if (!item) return;
    if (item.xhr) { try { item.xhr.abort(); } catch {} item.xhr = null; }
    removeFromQueue(id);
    showToast('info', t('cancelled') + (item.path || item.file.name));
  }

  function pauseQueueGroup(groupKey) {
    queue.forEach(item => {
      if (queueGroupKey(item) !== groupKey || item.status === 'done' || item.status === 'paused') return;
      item.status = 'paused';
      item.progress = 0;
      if (item.xhr) { try { item.xhr.abort(); } catch {} item.xhr = null; }
    });
    renderQueue();
  }

  function resumeQueueGroup(groupKey) {
    queue.forEach(item => {
      if (queueGroupKey(item) === groupKey && item.status === 'paused') {
        item.status = 'waiting';
        item.progress = 0;
      }
    });
    renderQueue();
    startOrQueueUpload();
  }

  function cancelQueueGroup(groupKey) {
    queue.forEach(item => {
      if (queueGroupKey(item) === groupKey && item.xhr) {
        try { item.xhr.abort(); } catch {}
        item.xhr = null;
      }
    });
    queue = queue.filter(item => queueGroupKey(item) !== groupKey || item.status === 'done');
    if (queue.length === 0) queueSection.style.display = 'none';
    else renderQueue();
    showToast('info', t('cancelled') + groupKey);
  }

  function retryUpload(id) {
    const item = queue.find(q => q.id === id);
    if (!item || item.status !== 'error') return;
    item.status = 'waiting';
    item.progress = 0;
    renderQueue();
    startOrQueueUpload();
  }

  queueClear.addEventListener('click', () => {
    queue = queue.filter(q => q.status !== 'done');
    renderQueue();
  });

  uploadAllBtn.addEventListener('click', uploadAll);

  function startOrQueueUpload() {
    if (isUploading) {
      uploadRequested = true;
      return;
    }
    uploadAll();
  }

  async function uploadAll() {
    if (isUploading) return;
    isUploading = true;
    const pending = queue.filter(q => q.status === 'waiting' || q.status === 'error');

    for (const item of pending) {
      if (!isConnected) break;
      if (!queue.includes(item) || (item.status !== 'waiting' && item.status !== 'error')) continue;
      item.status = 'uploading';
      item.progress = 0;
      renderQueue();

      try {
        await uploadSingle(item);
        if (item.status === 'uploading') item.status = 'done';
      } catch {
        if (item.status === 'uploading') item.status = 'error';
      }
      renderQueue();
    }

    isUploading = false;
    renderQueue();

    const doneCount = pending.filter(q => q.status === 'done').length;
    const failCount = pending.filter(q => q.status === 'error').length;
    if (failCount === 0 && doneCount > 0) {
      showToast('success', doneCount + t('uploadComplete'));
      loadFiles();
      setTimeout(() => {
        const allDone = queue.every(q => q.status === 'done');
        if (allDone) {
          queue = [];
          renderQueue();
        }
      }, 2000);
    } else if (doneCount > 0) {
      showToast('warning', doneCount + t('successCount') + failCount + t('failCount'));
      loadFiles();
    } else if (failCount > 0) {
      showToast('error', failCount + t('uploadFailed'));
    }

    if (uploadRequested && queue.some(q => q.status === 'waiting' || q.status === 'error')) {
      uploadRequested = false;
      uploadAll();
    } else {
      uploadRequested = false;
    }
  }

  async function uploadSingle(item) {
    item.uploadId = item.uploadId || uploadIdFor(item);
    const statusParams = new URLSearchParams({ token, uploadId: item.uploadId });
    let uploaded = 0;
    try {
      const status = await fetch('/api/upload/status?' + statusParams.toString());
      if (status.ok) {
        const data = await status.json();
        uploaded = Math.min(Number(data.uploadedBytes) || 0, item.file.size);
      }
    } catch {}

    item.uploadedBytes = uploaded;
    item.progress = item.file.size > 0 ? Math.round(uploaded / item.file.size * 100) : 0;
    item.startedAt = Date.now();
    item.speed = 0;
    item.eta = '';
    updateQueueProgress(item);

    while (uploaded < item.file.size) {
      if (item.status !== 'uploading') throw new Error('paused');
      const end = Math.min(uploaded + CHUNK_SIZE, item.file.size);
      const controller = new AbortController();
      item.xhr = controller;
      const params = new URLSearchParams({
        token,
        clientId,
        uploadId: item.uploadId,
        path: item.path || item.file.name,
        fileSize: String(item.file.size),
        offset: String(uploaded)
      });
      const response = await fetch('/api/upload/chunk?' + params.toString(), {
        method: 'POST',
        body: item.file.slice(uploaded, end),
        signal: controller.signal
      });
      item.xhr = null;
      if (!response.ok) throw new Error('Upload failed: ' + response.status);
      const data = await response.json();
      uploaded = Math.min(Number(data.uploadedBytes) || end, item.file.size);
      item.uploadedBytes = uploaded;
      item.progress = item.file.size > 0 ? Math.round(uploaded / item.file.size * 100) : 100;
      const elapsed = Math.max(1, (Date.now() - item.startedAt) / 1000);
      item.speed = uploaded / elapsed;
      item.eta = item.speed > 0 && uploaded < item.file.size ? formatDuration((item.file.size - uploaded) / item.speed) : '';
      if (data.completed) {
        item.checksum = data.checksum || '';
        item.verified = true;
      }
      updateQueueProgress(item);
    }

    item.verified = true;
    item.progress = 100;
    updateQueueProgress(item);
  }

  function updateQueueProgress(item) {
    const el = queueList.querySelector('[data-qid="' + item.id + '"]');
    if (!el) return;
    const bar = el.querySelector('.queue-progress-bar');
    if (bar) bar.style.width = item.progress + '%';
    const statusEl = el.querySelector('.queue-item-status');
    if (statusEl) statusEl.textContent = item.progress + '%';
    const speedEl = el.querySelector('.queue-speed');
    if (speedEl) speedEl.textContent = formatUploadMeta(item);
  }

  function uploadIdFor(item) {
    return simpleHash([item.path || item.file.name, item.file.size, item.file.lastModified].join('|'));
  }

  function simpleHash(value) {
    let hash = 2166136261;
    for (let i = 0; i < value.length; i++) {
      hash ^= value.charCodeAt(i);
      hash = Math.imul(hash, 16777619);
    }
    return Math.abs(hash >>> 0).toString(36);
  }

  function formatUploadMeta(item) {
    const parts = [formatSize(item.file.size)];
    if (item.speed) parts.push(formatSize(item.speed) + '/s');
    if (item.eta) parts.push(t('eta') + item.eta);
    if (item.verified) parts.push(t('verified'));
    return parts.join(' · ');
  }

  function formatDuration(seconds) {
    const total = Math.max(0, Math.round(seconds));
    const minutes = Math.floor(total / 60);
    const secs = total % 60;
    return minutes > 0 ? minutes + 'm ' + secs + 's' : secs + 's';
  }

  const expandedFolders = new Set();
  const folderChildren = new Map();
  const loadingFolders = new Set();

  function apiPath(path) {
    return String(path || '').split('/').filter(Boolean).map(encodeURIComponent).join('/');
  }

  function previewUrl(path) {
    const params = new URLSearchParams({ token, clientId });
    return '/api/preview/' + apiPath(path) + '?' + params.toString();
  }

  function downloadUrl(path) {
    const params = new URLSearchParams({ token, clientId });
    return '/api/download/' + apiPath(path) + '?' + params.toString();
  }

  async function fetchFiles(path) {
    const params = new URLSearchParams({ token });
    if (path) params.set('path', path);
    const r = await fetch('/api/files?' + params.toString());
    if (!r.ok) throw new Error('List failed');
    return r.json();
  }

  async function loadFiles() {
    filesRefresh.classList.add('loading');
    refreshBtn.classList.add('loading');
    try {
      const files = await fetchFiles('');
      expandedFolders.clear();
      folderChildren.clear();
      selectedFilePaths.clear();
      renderFiles(files);
      loadHistory();
    } catch {
      renderFiles([]);
    } finally {
      filesRefresh.classList.remove('loading');
      refreshBtn.classList.remove('loading');
    }
  }

  let currentFiles = [];

  function renderFiles(files) {
    currentFiles = files;
    const visibleFiles = applyFileView(files);
    filesCount.textContent = visibleFiles.length;
    filesDownloadAll.style.display = visibleFiles.length > 0 ? '' : 'none';
    updateBulkBar();

    if (visibleFiles.length === 0) {
      filesList.innerHTML = '';
      filesEmpty.style.display = '';
      filesClearAll.style.display = 'none';
      return;
    }
    filesEmpty.style.display = 'none';
    filesList.innerHTML = '';

    filesClearAll.style.display = files.length > 0 ? '' : 'none';
    renderFileRows(visibleFiles, 0);
  }

  function renderFileRows(files, level) {
    applyFileView(files).forEach(f => {
      const isDirectory = !!f.isDirectory;
      const itemPath = f.path || f.name;
      const type = isDirectory ? 'folder' : fileType(f.name);
      const expanded = isDirectory && expandedFolders.has(itemPath);
      const loading = isDirectory && loadingFolders.has(itemPath);
      const el = document.createElement('div');
      el.className = 'file-item' + (isDirectory ? ' folder-row' : '');
      el.style.paddingLeft = (16 + level * 18) + 'px';
      el.innerHTML =
        '<input class="file-check" type="checkbox" data-path="' + esc(itemPath) + '"' + (selectedFilePaths.has(itemPath) ? ' checked' : '') + '>' +
        (isDirectory
          ? '<button class="folder-toggle" data-path="' + esc(itemPath) + '" title="' + t('toggleFolder') + '">' +
              '<svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">' +
                (expanded ? '<path d="m6 9 6 6 6-6"/>' : '<path d="m9 18 6-6-6-6"/>') +
              '</svg>' +
            '</button>'
          : '<span class="folder-toggle-placeholder"></span>') +
        '<div class="file-icon ' + type + '">' + fileIconSVG(f.name, type) + '</div>' +
        '<div class="file-info">' +
          '<div class="file-name">' + esc(f.name) + '</div>' +
          '<div class="file-size">' + (isDirectory ? (loading ? t('loading') : t('folder')) : formatSize(f.size)) + '</div>' +
        '</div>' +
        '<button class="file-del" data-path="' + esc(itemPath) + '" data-name="' + esc(f.name) + '" title="' + t('delete') + '">' +
          '<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>' +
        '</button>' +
        '<button class="file-dl" data-path="' + esc(itemPath) + '" data-name="' + esc(f.name) + '" title="' + t('download') + '">' +
          '<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>' +
        '</button>';
      filesList.appendChild(el);
      el.querySelector('.file-check').addEventListener('click', e => {
        e.stopPropagation();
        if (e.currentTarget.checked) selectedFilePaths.add(itemPath);
        else selectedFilePaths.delete(itemPath);
        updateBulkBar();
      });

      const toggle = el.querySelector('.folder-toggle');
      if (toggle) {
        toggle.addEventListener('click', e => {
          e.stopPropagation();
          toggleFolder(itemPath);
        });
      }
      if (!isDirectory) {
        el.addEventListener('click', () => openPreview(f));
      }
      el.querySelector('.file-dl').addEventListener('click', e => {
        e.stopPropagation();
        window.location.href = downloadUrl(itemPath);
      });
      el.querySelector('.file-del').addEventListener('click', e => {
        e.stopPropagation();
        deleteFile(itemPath, f.name);
      });

      if (isDirectory && expanded) {
        const children = applyFileView(folderChildren.get(itemPath) || []);
        renderFileRows(children, level + 1);
      }
    });
  }

  function applyFileView(files) {
    const query = filesSearch.value.trim().toLowerCase();
    const type = filesTypeFilter.value;
    const [field, direction] = filesSort.value.split('-');
    return files
      .filter(file => {
        if (query && !file.name.toLowerCase().includes(query) && !(file.path || '').toLowerCase().includes(query)) return false;
        if (type !== 'all' && !file.isDirectory && fileType(file.name) !== type) return false;
        return true;
      })
      .slice()
      .sort((a, b) => {
        if (a.isDirectory !== b.isDirectory) return a.isDirectory ? -1 : 1;
        let result = 0;
        if (field === 'size') result = (a.size || 0) - (b.size || 0);
        else if (field === 'time') result = (a.modified || 0) - (b.modified || 0);
        else result = a.name.toLowerCase().localeCompare(b.name.toLowerCase());
        return direction === 'desc' ? -result : result;
      });
  }

  function updateBulkBar() {
    const count = selectedFilePaths.size;
    filesBulk.classList.toggle('show', count > 0);
    filesBulkCount.textContent = t('selectedCount') + count + t('selectedSuffix');
  }

  async function toggleFolder(path) {
    if (expandedFolders.has(path)) {
      expandedFolders.delete(path);
      renderFiles(currentFiles);
      return;
    }
    expandedFolders.add(path);
    if (!folderChildren.has(path)) {
      loadingFolders.add(path);
      renderFiles(currentFiles);
      try {
        folderChildren.set(path, await fetchFiles(path));
      } catch {
        folderChildren.set(path, []);
      } finally {
        loadingFolders.delete(path);
      }
    }
    renderFiles(currentFiles);
  }

  async function deleteFile(path, name) {
    try {
      const params = new URLSearchParams({ token, clientId });
      const r = await fetch('/api/files/' + apiPath(path) + '?' + params.toString(), { method: 'DELETE' });
      if (r.ok) {
        showToast('success', t('deleted') + name);
        loadFiles();
      } else {
        showToast('error', t('deleteFailed'));
      }
    } catch {
      showToast('error', t('deleteFailed'));
    }
  }

  async function clearAllFiles() {
    const files = currentFiles.slice();
    let deleted = 0;
    for (const f of files) {
      try {
        const params = new URLSearchParams({ token, clientId });
        const r = await fetch('/api/files/' + apiPath(f.path || f.name) + '?' + params.toString(), { method: 'DELETE' });
        if (r.ok) deleted++;
      } catch {}
    }
    showToast('success', t('deletedCountPrefix') + deleted + t('deletedCountSuffix'));
    loadFiles();
  }

  filesDownloadAll.addEventListener('click', () => {
    if (currentFiles.length === 0) return;
    downloadPaths(applyFileView(currentFiles).map(f => f.path || f.name));
    showToast('info', t('downloadStart') + currentFiles.length + t('fileCountSuffix'));
  });

  filesClearAll.addEventListener('click', clearAllFiles);
  refreshBtn.addEventListener('click', loadFiles);
  filesRefresh.addEventListener('click', loadFiles);
  historyRefresh.addEventListener('click', loadHistory);
  filesSearch.addEventListener('input', () => renderFiles(currentFiles));
  filesTypeFilter.addEventListener('change', () => renderFiles(currentFiles));
  filesSort.addEventListener('change', () => renderFiles(currentFiles));
  filesSelectAll.addEventListener('click', () => {
    collectVisiblePaths(currentFiles).forEach(path => selectedFilePaths.add(path));
    renderFiles(currentFiles);
  });
  filesBulkDownload.addEventListener('click', () => {
    downloadPaths(Array.from(selectedFilePaths));
  });
  filesBulkDelete.addEventListener('click', async () => {
    const paths = Array.from(selectedFilePaths);
    let deleted = 0;
    for (const path of paths) {
      try {
        const params = new URLSearchParams({ token, clientId });
        const r = await fetch('/api/files/' + apiPath(path) + '?' + params.toString(), { method: 'DELETE' });
        if (r.ok) deleted++;
      } catch {}
    }
    selectedFilePaths.clear();
    showToast('success', t('deletedCountPrefix') + deleted + t('deletedCountSuffix'));
    loadFiles();
  });

  function downloadPaths(paths) {
    if (paths.length === 0) return;
    if (paths.length === 1) {
      window.location.href = downloadUrl(paths[0]);
      setTimeout(loadHistory, 800);
      return;
    }
    const params = new URLSearchParams({ token, paths: JSON.stringify(paths) });
    window.location.href = '/api/download-zip?' + params.toString();
    setTimeout(loadHistory, 800);
  }

  function collectVisiblePaths(files) {
    const paths = [];
    applyFileView(files).forEach(file => {
      paths.push(file.path || file.name);
      const children = folderChildren.get(file.path || file.name);
      if (children) paths.push(...collectVisiblePaths(children));
    });
    return paths;
  }

  previewClose.addEventListener('click', closePreview);
  previewOverlay.addEventListener('click', e => {
    if (e.target === previewOverlay) closePreview();
  });
  previewDownload.addEventListener('click', () => {
    if (!currentPreview) return;
    window.location.href = downloadUrl(currentPreview.path || currentPreview.name);
  });
  document.addEventListener('keydown', e => {
    if (e.key === 'Escape' && previewOverlay.classList.contains('show')) closePreview();
  });

  function openPreview(file) {
    if (!file || file.isDirectory) return;
    currentPreview = file;
    previewTitle.textContent = file.name;
    previewMeta.textContent = formatSize(file.size);
    previewOverlay.classList.add('show');
    renderPreview(file);
  }

  async function renderPreview(file) {
    const requestId = ++previewRequestId;
    const kind = previewKind(file.name);
    const url = previewUrl(file.path || file.name);
    showPreviewState('info', t('previewLoading'));

    if (kind === 'unsupported') {
      showPreviewState('info', t('unsupportedPreview'));
      return;
    }
    if (kind === 'text' && file.size > TEXT_PREVIEW_LIMIT) {
      showPreviewState('warning', t('fileTooLargePreview'));
      return;
    }

    try {
      if (kind === 'image') {
        previewBody.innerHTML = '<img class="preview-media" alt="">';
        const img = previewBody.querySelector('img');
        img.alt = file.name;
        img.src = url;
        return;
      }
      if (kind === 'video') {
        previewBody.innerHTML = '<video class="preview-video" controls playsinline></video>';
        previewBody.querySelector('video').src = url;
        return;
      }
      if (kind === 'audio') {
        previewBody.innerHTML = '<audio class="preview-audio" controls></audio>';
        previewBody.querySelector('audio').src = url;
        return;
      }
      if (kind === 'pdf') {
        previewBody.innerHTML = '<iframe class="preview-frame" title="' + esc(file.name) + '"></iframe>';
        previewBody.querySelector('iframe').src = url;
        return;
      }

      const r = await fetch(url);
      if (!r.ok) throw new Error('Preview failed');
      const text = await r.text();
      if (requestId !== previewRequestId) return;
      previewBody.innerHTML = '<pre class="preview-text"></pre>';
      previewBody.querySelector('pre').textContent = text;
    } catch {
      if (requestId !== previewRequestId) return;
      showPreviewState('error', t('previewFailed'));
    }
  }

  function closePreview() {
    previewRequestId++;
    previewOverlay.classList.remove('show');
    previewBody.innerHTML = '';
    previewTitle.textContent = t('preview');
    previewMeta.textContent = '';
    currentPreview = null;
  }

  function showPreviewState(type, message) {
    const icons = {
      info: '<svg xmlns="http://www.w3.org/2000/svg" width="36" height="36" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><path d="M12 16v-4"/><path d="M12 8h.01"/></svg>',
      warning: '<svg xmlns="http://www.w3.org/2000/svg" width="36" height="36" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3"/><path d="M12 9v4"/><path d="M12 17h.01"/></svg>',
      error: '<svg xmlns="http://www.w3.org/2000/svg" width="36" height="36" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><path d="M15 9 9 15"/><path d="m9 9 6 6"/></svg>'
    };
    previewBody.innerHTML = '<div class="preview-state">' + (icons[type] || icons.info) + '<div>' + esc(message) + '</div></div>';
  }

  function previewKind(name) {
    const ext = (name.split('.').pop() || '').toLowerCase();
    if (['jpg','jpeg','png','gif','webp','bmp','ico','avif'].includes(ext)) return 'image';
    if (['mp4','webm','mov','m4v'].includes(ext)) return 'video';
    if (['mp3','wav','ogg','m4a','flac'].includes(ext)) return 'audio';
    if (ext === 'pdf') return 'pdf';
    if (['txt','log','md','csv','json','jsonl','js','ts','jsx','tsx','css','html','htm','xml','svg','yaml','yml','toml','rs','py','go','java','c','cpp','h','hpp','sh','bat','ps1','sql'].includes(ext)) return 'text';
    return 'unsupported';
  }

  async function loadHistory() {
    if (!token) return;
    try {
      const r = await fetch('/api/history?token=' + encodeURIComponent(token));
      if (!r.ok) throw new Error('History failed');
      renderHistory(await r.json());
    } catch {
      renderHistory([]);
    }
  }

  function renderHistory(entries) {
    if (!entries || entries.length === 0) {
      historyList.innerHTML = '<div class="empty-state"><p>' + t('noFiles') + '</p></div>';
      return;
    }
    historyList.innerHTML = '';
    entries.slice(0, 30).forEach(entry => {
      const el = document.createElement('div');
      el.className = 'history-item';
      const typeLabel = entry.type === 'upload' ? t('uploading').replace('…','') : t('download');
      const time = entry.timestamp ? new Date(entry.timestamp).toLocaleString() : '';
      el.innerHTML =
        '<span class="history-badge ' + esc(entry.type || '') + '">' + esc(typeLabel) + '</span>' +
        '<div class="history-info">' +
          '<div class="history-name">' + esc(entry.name || entry.path || '') + '</div>' +
          '<div class="history-meta">' + esc(formatSize(entry.size || 0)) + (entry.checksum ? ' · SHA-256 ' + esc(entry.checksum.slice(0, 12)) : '') + (time ? ' · ' + esc(time) : '') + '</div>' +
        '</div>';
      historyList.appendChild(el);
    });
  }

  function fileType(name) {
    const ext = (name.split('.').pop() || '').toLowerCase();
    if (['jpg','jpeg','png','gif','webp','svg','bmp','ico','heic','avif'].includes(ext)) return 'img';
    if (['mp4','mov','avi','mkv','webm','flv','wmv'].includes(ext)) return 'video';
    if (['mp3','wav','flac','aac','ogg','m4a','wma'].includes(ext)) return 'audio';
    if (['zip','rar','7z','tar','gz','bz2','xz','apk'].includes(ext)) return 'archive';
    if (['js','ts','py','rs','go','java','c','cpp','h','rb','php','sh','html','css','json','yaml','yml','toml','xml','sql','md'].includes(ext)) return 'code';
    if (['pdf','doc','docx','xls','xlsx','ppt','pptx','txt','rtf','csv'].includes(ext)) return 'doc';
    return 'default';
  }

  function fileIconSVG(name, type) {
    const icons = {
      img: '<svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="18" height="18" x="3" y="3" rx="2" ry="2"/><circle cx="9" cy="9" r="2"/><path d="m21 15-3.086-3.086a2 2 0 0 0-2.828 0L6 21"/></svg>',
      video: '<svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m16 13 5.223 3.482a.5.5 0 0 0 .777-.416V7.87a.5.5 0 0 0-.752-.432L16 10.5"/><rect x="2" y="6" width="14" height="12" rx="2"/></svg>',
      audio: '<svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M9 18V5l12-2v13"/><circle cx="6" cy="18" r="3"/><circle cx="18" cy="16" r="3"/></svg>',
      doc: '<svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7Z"/><path d="M14 2v4a2 2 0 0 0 2 2h4"/><path d="M10 9H8"/><path d="M16 13H8"/><path d="M16 17H8"/></svg>',
      archive: '<svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="20" height="5" x="2" y="3" rx="1"/><path d="M4 8v11a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8"/><path d="M10 12h4"/></svg>',
      code: '<svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="16 18 22 12 16 6"/><polyline points="8 6 2 12 8 18"/></svg>',
      folder: '<svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z"/></svg>',
      default: '<svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7Z"/><path d="M14 2v4a2 2 0 0 0 2 2h4"/></svg>'
    };
    return icons[type] || icons.default;
  }

  function formatSize(b) {
    if (b < 1024) return b + ' B';
    if (b < 1048576) return (b / 1024).toFixed(1) + ' KB';
    if (b < 1073741824) return (b / 1048576).toFixed(1) + ' MB';
    return (b / 1073741824).toFixed(2) + ' GB';
  }

  function esc(s) {
    const d = document.createElement('div');
    d.textContent = s;
    return d.innerHTML;
  }

  let toastTimer;
  function showToast(type, msg) {
    clearTimeout(toastTimer);
    const icons = {
      success: '<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg>',
      error: '<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><line x1="15" y1="9" x2="9" y2="15"/><line x1="9" y1="9" x2="15" y2="15"/></svg>',
      warning: '<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3"/><path d="M12 9v4"/><path d="M12 17h.01"/></svg>',
      info: '<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><path d="M12 16v-4"/><path d="M12 8h.01"/></svg>'
    };
    toastEl.className = 'toast ' + type;
    toastEl.innerHTML = (icons[type] || '') + '<span>' + esc(msg) + '</span>';
    requestAnimationFrame(() => toastEl.classList.add('show'));
    toastTimer = setTimeout(() => toastEl.classList.remove('show'), 3000);
  }
})();
</script>
</body>
</html>
"##;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_filename_preserves_unicode_and_removes_dangerous_chars() {
        assert_eq!(sanitize_filename("中文 report?.txt"), "中文 report_.txt");
        assert_eq!(sanitize_filename("../bad\\name"), "_bad_name");
        assert_eq!(sanitize_filename("..."), "unknown");
    }

    #[test]
    fn percent_encode_header_value_encodes_unicode() {
        assert_eq!(
            percent_encode_header_value("中文 report.txt"),
            "%E4%B8%AD%E6%96%87%20report.txt"
        );
    }

    #[test]
    fn format_access_url_wraps_ipv6_hosts() {
        let ip = "::1".parse().unwrap();
        assert_eq!(format_access_url(ip, 37210), "http://[::1]:37210");
    }

    #[test]
    fn guess_content_type_covers_previewable_files() {
        assert_eq!(guess_content_type("photo.WEBP"), "image/webp");
        assert_eq!(guess_content_type("clip.webm"), "video/webm");
        assert_eq!(guess_content_type("sound.flac"), "audio/flac");
        assert_eq!(guess_content_type("report.pdf"), "application/pdf");
        assert_eq!(guess_content_type("notes.md"), "text/plain; charset=utf-8");
        assert_eq!(
            guess_content_type("bundle.apk"),
            "application/vnd.android.package-archive"
        );
        assert_eq!(guess_content_type("blob.bin"), "application/octet-stream");
    }

    #[test]
    fn preview_content_type_keeps_executable_markup_as_text() {
        assert_eq!(
            preview_content_type("index.html"),
            "text/plain; charset=utf-8"
        );
        assert_eq!(
            preview_content_type("vector.svg"),
            "text/plain; charset=utf-8"
        );
    }

    #[test]
    fn parse_range_header_supports_common_ranges() {
        assert_eq!(parse_range_header("bytes=0-99", 1000), Some((0, 99)));
        assert_eq!(parse_range_header("bytes=100-", 1000), Some((100, 999)));
        assert_eq!(parse_range_header("bytes=-200", 1000), Some((800, 999)));
        assert_eq!(parse_range_header("bytes=1000-1200", 1000), None);
    }
}
