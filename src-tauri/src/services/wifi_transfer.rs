use std::net::SocketAddr;
use std::path::PathBuf;

use axum::{
    body::Body,
    extract::{DefaultBodyLimit, Multipart, Path, Query, State},
    http::{header, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Json, Response},
    routing::{get, post},
    Router,
};
use base64::Engine;
use qrcode::QrCode;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;
use tokio::sync::watch;
use tokio_util::io::ReaderStream;

use crate::core::error::AppError;
use crate::core::types::WifiTransferStatus;
use crate::services::transfer::TransferService;

const DEFAULT_PORT: u16 = 37210;
const MAX_UPLOAD_BYTES: usize = 2 * 1024 * 1024 * 1024;

#[derive(Clone)]
struct AppState {
    token: String,
    upload_dir: PathBuf,
    app_handle: tauri::AppHandle,
}

#[derive(Deserialize)]
struct VerifyQuery {
    token: Option<String>,
}

#[derive(Serialize)]
struct FileInfo {
    name: String,
    size: u64,
}

pub async fn start_server(
    transfer_service: &TransferService,
    port: Option<u16>,
) -> Result<WifiTransferStatus, AppError> {
    let current = transfer_service.get_wifi_transfer_status();
    if current.running {
        return Ok(current);
    }

    let port = port.unwrap_or(DEFAULT_PORT);
    let token = nanoid::nanoid!(8);
    let upload_dir = std::env::temp_dir().join("devicedeck-wifi-transfer");
    tokio::fs::create_dir_all(&upload_dir).await?;

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

    let state = AppState {
        token: token.clone(),
        upload_dir: upload_dir.clone(),
        app_handle: transfer_service.app_handle(),
    };

    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    let app = Router::new()
        .route("/", get(serve_mobile_page))
        .route("/api/verify", get(verify_token))
        .route("/api/upload", post(upload_file))
        .route("/api/files", get(list_files))
        .route("/api/download/{name}", get(download_file))
        .layer(DefaultBodyLimit::max(MAX_UPLOAD_BYTES))
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

    transfer_service.update_wifi_status(WifiTransferStatus {
        running: false,
        url: None,
        token: None,
        qr_code_data_url: None,
        port: current.port,
    });

    Ok(())
}

async fn serve_mobile_page() -> Html<&'static str> {
    Html(MOBILE_HTML)
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

    while let Ok(Some(field)) = multipart.next_field().await {
        let file_name = field.file_name().unwrap_or("unknown").to_string();
        let dest = unique_upload_path(&state.upload_dir, &sanitize_filename(&file_name)).await;
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
    }

    (StatusCode::OK, Json(serde_json::json!({ "ok": true })))
}

fn format_access_url(ip: std::net::IpAddr, port: u16) -> String {
    match ip {
        std::net::IpAddr::V4(ip) => format!("http://{}:{}", ip, port),
        std::net::IpAddr::V6(ip) => format!("http://[{}]:{}", ip, port),
    }
}

async fn unique_upload_path(upload_dir: &std::path::Path, file_name: &str) -> PathBuf {
    let candidate = upload_dir.join(file_name);
    if tokio::fs::metadata(&candidate).await.is_err() {
        return candidate;
    }

    let path = std::path::Path::new(file_name);
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("file");
    let extension = path.extension().and_then(|s| s.to_str());
    for index in 1..1000 {
        let name = match extension {
            Some(extension) if !extension.is_empty() => format!("{stem} ({index}).{extension}"),
            _ => format!("{stem} ({index})"),
        };
        let candidate = upload_dir.join(name);
        if tokio::fs::metadata(&candidate).await.is_err() {
            return candidate;
        }
    }
    upload_dir.join(format!("{}-{}", nanoid::nanoid!(6), file_name))
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

async fn list_files(
    State(state): State<AppState>,
    Query(query): Query<VerifyQuery>,
) -> impl IntoResponse {
    if query.token.as_deref() != Some(&state.token) {
        return (StatusCode::UNAUTHORIZED, Json(Vec::<FileInfo>::new()));
    }

    let mut files = Vec::new();
    if let Ok(mut entries) = tokio::fs::read_dir(&state.upload_dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            if let Ok(metadata) = entry.metadata().await {
                if metadata.is_file() {
                    if let Some(name) = entry.file_name().to_str() {
                        files.push(FileInfo {
                            name: name.to_string(),
                            size: metadata.len(),
                        });
                    }
                }
            }
        }
    }

    files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    (StatusCode::OK, Json(files))
}

async fn download_file(
    State(state): State<AppState>,
    Query(query): Query<VerifyQuery>,
    Path(name): Path<String>,
) -> Response {
    if query.token.as_deref() != Some(&state.token) {
        return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
    }

    let sanitized = sanitize_filename(&name);
    let path = state.upload_dir.join(&sanitized);

    match tokio::fs::File::open(&path).await {
        Ok(file) => {
            let stream = ReaderStream::new(file);
            let body = Body::from_stream(stream);
            let disposition = format!(
                "attachment; filename*=UTF-8''{}",
                percent_encode_header_value(&sanitized)
            );
            let mut response = Response::new(body);
            response.headers_mut().insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static(guess_content_type(&sanitized)),
            );
            if let Ok(value) = HeaderValue::from_str(&disposition) {
                response
                    .headers_mut()
                    .insert(header::CONTENT_DISPOSITION, value);
            }
            response
        }
        Err(_) => (StatusCode::NOT_FOUND, "File not found").into_response(),
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
    match name.rsplit('.').next().unwrap_or("") {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "pdf" => "application/pdf",
        "zip" => "application/zip",
        "apk" => "application/vnd.android.package-archive",
        "mp4" => "video/mp4",
        "mp3" => "audio/mpeg",
        _ => "application/octet-stream",
    }
}

const MOBILE_HTML: &str = r##"<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width,initial-scale=1,maximum-scale=1">
<title>DeviceDeck File Transfer</title>
<style>
*{box-sizing:border-box;margin:0;padding:0}
body{font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif;background:#0f172a;color:#e2e8f0;min-height:100vh;padding:20px}
.container{max-width:480px;margin:0 auto}
h1{text-align:center;font-size:20px;margin-bottom:24px;color:#60a5fa}
.card{background:#1e293b;border-radius:12px;padding:20px;margin-bottom:16px}
label{display:block;font-size:14px;margin-bottom:8px;color:#94a3b8}
input[type=text]{width:100%;padding:12px;border:1px solid #334155;border-radius:8px;background:#0f172a;color:#e2e8f0;font-size:16px;outline:none}
input[type=text]:focus{border-color:#60a5fa}
button{width:100%;padding:12px;border:none;border-radius:8px;font-size:16px;cursor:pointer;transition:all 0.2s}
.btn-primary{background:#3b82f6;color:white}
.btn-primary:hover{background:#2563eb}
.btn-primary:disabled{background:#475569;cursor:not-allowed}
.upload-area{border:2px dashed #334155;border-radius:12px;padding:32px;text-align:center;cursor:pointer;transition:border-color 0.2s;margin-bottom:16px}
.upload-area:hover,.upload-area.dragover{border-color:#3b82f6}
.upload-area p{color:#94a3b8;font-size:14px}
.files{list-style:none}
.files li{display:flex;justify-content:space-between;align-items:center;padding:10px 0;border-bottom:1px solid #1e293b}
.files li:last-child{border-bottom:none}
.file-name{flex:1;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;margin-right:12px;font-size:14px}
.file-size{color:#64748b;font-size:12px;margin-right:12px}
.btn-dl{padding:6px 16px;background:transparent;border:1px solid #3b82f6;color:#3b82f6;border-radius:6px;font-size:13px;cursor:pointer}
.btn-dl:hover{background:#3b82f620}
.hidden{display:none}
.toast{position:fixed;top:20px;left:50%;transform:translateX(-50%);background:#22c55e;color:white;padding:10px 24px;border-radius:8px;font-size:14px;z-index:1000;opacity:0;transition:opacity 0.3s}
.toast.show{opacity:1}
</style>
</head>
<body>
<div class="container">
<h1>DeviceDeck File Transfer</h1>

<div id="auth" class="card">
  <label>Enter Token</label>
  <input type="text" id="token-input" placeholder="8-digit code" maxlength="8">
  <button class="btn-primary" id="verify-btn" style="margin-top:12px">Verify</button>
</div>

<div id="main" class="hidden">
  <div class="card">
    <div class="upload-area" id="upload-area">
      <p>Click or drag files here to upload</p>
      <input type="file" id="file-input" multiple style="display:none">
    </div>
    <button class="btn-primary" id="upload-btn" disabled>Upload</button>
  </div>
  <div class="card">
    <h2 style="font-size:16px;margin-bottom:12px">Download Files</h2>
    <ul class="files" id="file-list"></ul>
    <p id="no-files" style="color:#64748b;font-size:14px;text-align:center">No files available</p>
  </div>
</div>
</div>
<div class="toast" id="toast"></div>

<script>
const token=new URLSearchParams(location.search).get('token')||'';
let verified=false;
const filesToUpload=[];

if(token){
  document.getElementById('token-input').value=token;
  verify();
}

document.getElementById('verify-btn').onclick=verify;
document.getElementById('file-input').onchange=e=>{addFiles(e.target.files)};
document.getElementById('upload-area').onclick=()=>document.getElementById('file-input').click();
document.getElementById('upload-area').ondragover=e=>{e.preventDefault();e.currentTarget.classList.add('dragover')};
document.getElementById('upload-area').ondragleave=e=>{e.currentTarget.classList.remove('dragover')};
document.getElementById('upload-area').ondrop=e=>{e.preventDefault();e.currentTarget.classList.remove('dragover');addFiles(e.dataTransfer.files)};
document.getElementById('upload-btn').onclick=upload;

function addFiles(fileList){
  for(const f of fileList) filesToUpload.push(f);
  updateUploadBtn();
  if(filesToUpload.length>0){
    document.querySelector('.upload-area p').textContent=filesToUpload.length+' file(s) selected';
  }
}

function updateUploadBtn(){
  document.getElementById('upload-btn').disabled=filesToUpload.length===0;
}

async function verify(){
  const t=document.getElementById('token-input').value.trim();
  if(!t)return;
  const r=await fetch('/api/verify?token='+encodeURIComponent(t));
  const d=await r.json();
  if(d.valid){
    verified=true;
    document.getElementById('auth').classList.add('hidden');
    document.getElementById('main').classList.remove('hidden');
    loadFiles();
  }else{
    showToast('Invalid token');
  }
}

async function upload(){
  if(!verified||filesToUpload.length===0)return;
  const btn=document.getElementById('upload-btn');
  btn.disabled=true;btn.textContent='Uploading...';
  const t=document.getElementById('token-input').value.trim();
  const fd=new FormData();
  for(const f of filesToUpload) fd.append('files',f);
  try{
    const r=await fetch('/api/upload?token='+encodeURIComponent(t),{method:'POST',body:fd});
    if(r.ok){
      showToast('Upload complete!');
      filesToUpload.length=0;
      document.querySelector('.upload-area p').textContent='Click or drag files here to upload';
      updateUploadBtn();
      loadFiles();
    }else{
      showToast('Upload failed');
    }
  }catch(e){
    showToast('Network error');
  }
  btn.disabled=false;btn.textContent='Upload';
}

async function loadFiles(){
  const t=document.getElementById('token-input').value.trim();
  const r=await fetch('/api/files?token='+encodeURIComponent(t));
  const files=await r.json();
  const list=document.getElementById('file-list');
  const noFiles=document.getElementById('no-files');
  list.innerHTML='';
  if(files.length===0){noFiles.classList.remove('hidden');return;}
  noFiles.classList.add('hidden');
  for(const f of files){
    const li=document.createElement('li');
    const name=document.createElement('span');
    name.className='file-name';
    name.textContent=f.name;
    const size=document.createElement('span');
    size.className='file-size';
    size.textContent=formatSize(f.size);
    const btn=document.createElement('button');
    btn.className='btn-dl';
    btn.textContent='Download';
    btn.onclick=()=>downloadFile(f.name);
    li.append(name,size,btn);
    list.appendChild(li);
  }
}

function downloadFile(name){
  const t=document.getElementById('token-input').value.trim();
  window.location.href='/api/download/'+encodeURIComponent(name)+'?token='+encodeURIComponent(t);
}

function formatSize(b){
  if(b<1024)return b+'B';
  if(b<1048576)return(b/1024).toFixed(1)+'KB';
  return(b/1048576).toFixed(1)+'MB';
}

function showToast(msg){
  const t=document.getElementById('toast');
  t.textContent=msg;t.classList.add('show');
  setTimeout(()=>t.classList.remove('show'),2000);
}
</script>
</body>
</html>"##;

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
}
