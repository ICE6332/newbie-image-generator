use axum::{
    extract::{Path, Query, State, WebSocketUpgrade},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use futures::{SinkExt, StreamExt};
use reqwest::Url;
use serde::Deserialize;
use std::net::{IpAddr, ToSocketAddrs};
use std::sync::Arc;
use tokio::sync::broadcast;
use tower_http::services::{ServeDir, ServeFile};
use uuid::Uuid;

use crate::comfyui::ComfyUIClient;
use crate::config::{ComfyUIUrlPolicy, Config};
use crate::error::{AppError, AppResult};
use crate::models::*;
use crate::services::prompt_optimizer::PromptOptimizer;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub comfyui: ComfyUIClient,
    pub event_tx: broadcast::Sender<String>,
    pub comfyui_client_id: String,
    pub prompt_optimizer: PromptOptimizer,
    pub config: Arc<Config>,
}

/// Create the API router
pub fn create_router(state: AppState) -> Router {
    // Serve frontend static files - check both ./dist and ./frontend/dist
    let dist_path = if std::path::Path::new("dist").exists() {
        "dist"
    } else if std::path::Path::new("frontend/dist").exists() {
        "frontend/dist"
    } else {
        "dist" // fallback
    };
    let index_path = format!("{}/index.html", dist_path);
    let serve_dir = ServeDir::new(dist_path).fallback(ServeFile::new(&index_path));

    Router::new()
        // Health and status endpoints
        .route("/health", get(health_handler))
        .route("/api/health", get(health_handler))
        .route("/api/status", get(status_handler))
        .route("/api/test-comfyui", post(test_comfyui_handler))
        .route(
            "/api/comfyui-url",
            get(get_comfyui_url_handler).post(set_comfyui_url_handler),
        )
        // Generation endpoints
        .route("/api/generate", post(generate_handler))
        .route("/api/prompt/optimize", post(optimize_prompt_handler))
        .route("/api/queue", get(queue_handler))
        .route("/api/history/{prompt_id}", get(history_handler))
        // Image endpoints
        .route("/api/images/*filename", get(image_handler))
        // Control endpoints
        .route("/api/interrupt", post(interrupt_handler))
        .route("/api/clear", post(clear_handler))
        // WebSocket endpoint
        .route("/ws", get(websocket_handler))
        .with_state(state)
        // Fallback to serve static files (frontend)
        .fallback_service(serve_dir)
}

// ============================================================================
// Prompt Optimization Handlers
// ============================================================================

async fn optimize_prompt_handler(
    State(state): State<AppState>,
    Json(request): Json<OptimizePromptRequest>,
) -> AppResult<Json<OptimizePromptResponse>> {
    let resp = state.prompt_optimizer.optimize(request).await?;
    Ok(Json(resp))
}

fn normalize_comfyui_url(raw: &str, policy: ComfyUIUrlPolicy) -> AppResult<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(AppError::InvalidRequest(
            "ComfyUI URL cannot be empty".to_string(),
        ));
    }

    let candidate = trimmed.trim_end_matches('/');
    let parsed = if candidate.starts_with("http://") || candidate.starts_with("https://") {
        Url::parse(candidate)
    } else {
        Url::parse(&format!("http://{}", candidate))
    }
    .map_err(|_| AppError::InvalidRequest("Invalid ComfyUI URL".to_string()))?;

    match parsed.scheme() {
        "http" | "https" => {}
        _ => {
            return Err(AppError::InvalidRequest(
                "ComfyUI URL must start with http:// or https://".to_string(),
            ))
        }
    }

    if parsed.path() != "" && parsed.path() != "/" {
        return Err(AppError::InvalidRequest(
            "ComfyUI URL must not include a path".to_string(),
        ));
    }

    let host = parsed
        .host_str()
        .ok_or_else(|| AppError::InvalidRequest("ComfyUI URL missing host".to_string()))?;

    if !is_allowed_comfyui_host(host, parsed.port_or_known_default(), policy) {
        return Err(AppError::InvalidRequest(format!(
            "ComfyUI URL must be local/LAN. Set ALLOW_REMOTE_COMFYUI=true to allow public hosts. (current={:?})",
            policy
        )));
    }

    Ok(parsed.as_str().trim_end_matches('/').to_string())
}

fn is_allowed_comfyui_host(host: &str, port: Option<u16>, policy: ComfyUIUrlPolicy) -> bool {
    if policy == ComfyUIUrlPolicy::Any {
        return true;
    }

    if is_loopback_host(host) {
        return true;
    }

    if policy == ComfyUIUrlPolicy::LocalOnly {
        return false;
    }

    let port = port.unwrap_or(80);
    is_lan_host(host, port)
}

fn is_loopback_host(host: &str) -> bool {
    matches!(host, "localhost" | "127.0.0.1" | "::1")
}

fn is_lan_host(host: &str, port: u16) -> bool {
    if let Ok(ip) = host.parse::<IpAddr>() {
        return is_lan_ip(ip);
    }

    let addrs = (host, port).to_socket_addrs();
    let mut saw_addr = false;

    if let Ok(iter) = addrs {
        for addr in iter {
            saw_addr = true;
            if !is_lan_ip(addr.ip()) {
                return false;
            }
        }
    }

    saw_addr
}

fn is_lan_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => v4.is_private() || v4.is_loopback() || v4.is_link_local() || is_cgnat(v4),
        IpAddr::V6(v6) => v6.is_loopback() || v6.is_unique_local() || v6.is_unicast_link_local(),
    }
}

fn is_cgnat(ip: std::net::Ipv4Addr) -> bool {
    let octets = ip.octets();
    octets[0] == 100 && (octets[1] & 0b1100_0000) == 0b0100_0000
}

// ============================================================================
// Health and Status Handlers
// ============================================================================

async fn health_handler(State(state): State<AppState>) -> AppResult<Json<serde_json::Value>> {
    let comfyui_ok = state.comfyui.health_check().await?;

    Ok(Json(serde_json::json!({
        "status": if comfyui_ok { "ok" } else { "degraded" },
        "comfyui": comfyui_ok
    })))
}

#[derive(Deserialize)]
struct TestComfyUIRequest {
    url: String,
}

async fn test_comfyui_handler(
    State(state): State<AppState>,
    Json(request): Json<TestComfyUIRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let url = normalize_comfyui_url(&request.url, state.config.comfyui_url_policy)?;
    let test_url = format!("{}/system_stats", url);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap();

    let success = match client.get(&test_url).send().await {
        Ok(resp) if resp.status().is_success() => true,
        _ => false,
    };

    Ok(Json(serde_json::json!({ "success": success })))
}

async fn get_comfyui_url_handler(State(state): State<AppState>) -> Json<serde_json::Value> {
    let url = state.comfyui.get_url().await;
    Json(serde_json::json!({ "url": url }))
}

#[derive(Deserialize)]
struct SetComfyUIUrlRequest {
    url: String,
}

async fn set_comfyui_url_handler(
    State(state): State<AppState>,
    Json(request): Json<SetComfyUIUrlRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let url = normalize_comfyui_url(&request.url, state.config.comfyui_url_policy)?;
    state.comfyui.set_url(&url).await;
    tracing::info!("ComfyUI URL updated to: {}", url);
    Ok(Json(serde_json::json!({ "success": true, "url": url })))
}

async fn status_handler(State(state): State<AppState>) -> AppResult<Json<serde_json::Value>> {
    let system_stats = state.comfyui.get_system_stats().await?;
    let queue = state.comfyui.get_queue().await?;

    Ok(Json(serde_json::json!({
        "comfyui": {
            "connected": true,
            "system": system_stats.system,
            "devices": system_stats.devices
        },
        "queue": {
            "running": queue.queue_running.len(),
            "pending": queue.queue_pending.len()
        }
    })))
}

// ============================================================================
// Generation Handlers
// ============================================================================

async fn generate_handler(
    State(state): State<AppState>,
    Json(request): Json<GenerateRequest>,
) -> AppResult<Json<QueueResponse>> {
    tracing::info!(
        "Generate request: prompt='{}', size={}x{}, steps={}",
        request.prompt.chars().take(50).collect::<String>(),
        request.width,
        request.height,
        request.steps
    );

    // Validate request
    if request.prompt.trim().is_empty() {
        return Err(AppError::InvalidRequest(
            "Prompt cannot be empty".to_string(),
        ));
    }

    if request.width < 64 || request.width > 4096 {
        return Err(AppError::InvalidRequest(
            "Width must be between 64 and 4096".to_string(),
        ));
    }

    if request.height < 64 || request.height > 4096 {
        return Err(AppError::InvalidRequest(
            "Height must be between 64 and 4096".to_string(),
        ));
    }

    // Get available models and build workflow
    let models = state.comfyui.get_available_models().await?;
    let workflow = state.comfyui.build_workflow(&request, &models);

    // Queue the prompt with the backend's ComfyUI client_id so we receive events
    let response = state
        .comfyui
        .queue_prompt(workflow, Some(state.comfyui_client_id.clone()))
        .await?;

    tracing::info!(
        "Prompt queued: id={}, number={}",
        response.prompt_id,
        response.number
    );

    Ok(Json(QueueResponse {
        prompt_id: response.prompt_id,
        number: response.number,
    }))
}

async fn queue_handler(State(state): State<AppState>) -> AppResult<Json<serde_json::Value>> {
    let queue = state.comfyui.get_queue().await?;

    Ok(Json(serde_json::json!({
        "running": queue.queue_running.len(),
        "pending": queue.queue_pending.len(),
        "running_prompts": queue.queue_running,
        "pending_prompts": queue.queue_pending
    })))
}

async fn history_handler(
    State(state): State<AppState>,
    Path(prompt_id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    let history = state.comfyui.get_history(&prompt_id).await?;

    match history {
        Some(h) => {
            // Collect all images from outputs (metadata only)
            let images: Vec<ImageResult> = h
                .outputs
                .values()
                .flat_map(|output| {
                    output.images.iter().filter_map(|img| {
                        if img.image_type != "output" {
                            return None;
                        }
                        Some(ImageResult {
                            filename: img.filename.clone(),
                            subfolder: img.subfolder.clone(),
                            image_type: img.image_type.clone(),
                        })
                    })
                })
                .collect();

            Ok(Json(serde_json::json!({
                "prompt_id": prompt_id,
                "status": h.status.status_str.unwrap_or_else(|| "unknown".to_string()),
                "completed": h.status.completed.unwrap_or(false),
                "images": images
            })))
        }
        None => Err(AppError::NotFound(format!(
            "Prompt {} not found",
            prompt_id
        ))),
    }
}

// ============================================================================
// Image Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ImageQuery {
    subfolder: Option<String>,
    #[serde(rename = "type")]
    image_type: Option<String>,
}

async fn image_handler(
    State(state): State<AppState>,
    Path(filename): Path<String>,
    Query(query): Query<ImageQuery>,
) -> AppResult<impl IntoResponse> {
    let subfolder = query.subfolder.unwrap_or_default();
    let image_type = query.image_type.unwrap_or_else(|| "output".to_string());

    let image_data = state
        .comfyui
        .get_image(&filename, &subfolder, &image_type)
        .await?;

    // Determine content type based on filename
    let content_type = if filename.ends_with(".png") {
        "image/png"
    } else if filename.ends_with(".jpg") || filename.ends_with(".jpeg") {
        "image/jpeg"
    } else if filename.ends_with(".webp") {
        "image/webp"
    } else {
        "application/octet-stream"
    };

    Ok((
        [(axum::http::header::CONTENT_TYPE, content_type)],
        image_data,
    ))
}

// ============================================================================
// Control Handlers
// ============================================================================

async fn interrupt_handler(State(state): State<AppState>) -> AppResult<Json<serde_json::Value>> {
    state.comfyui.interrupt().await?;
    tracing::info!("Execution interrupted");

    Ok(Json(serde_json::json!({
        "status": "interrupted"
    })))
}

async fn clear_handler(State(state): State<AppState>) -> AppResult<Json<serde_json::Value>> {
    state.comfyui.clear_queue().await?;
    tracing::info!("Queue cleared");

    Ok(Json(serde_json::json!({
        "status": "cleared"
    })))
}

// ============================================================================
// WebSocket Handler
// ============================================================================

async fn websocket_handler(
    State(state): State<AppState>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_websocket(socket, state))
}

async fn handle_websocket(socket: axum::extract::ws::WebSocket, state: AppState) {
    let client_id = Uuid::new_v4().to_string();
    tracing::info!("WebSocket connected: {}", client_id);

    let (mut sender, mut receiver) = socket.split();

    // Send connected message
    let connected_msg = FrontendMessage::Connected {
        client_id: client_id.clone(),
    };
    if let Ok(msg) = serde_json::to_string(&connected_msg) {
        let _ = sender.send(axum::extract::ws::Message::Text(msg)).await;
    }

    // Subscribe to events
    let mut event_rx = state.event_tx.subscribe();

    // Spawn task to forward events to client
    let forward_task = tokio::spawn(async move {
        while let Ok(event) = event_rx.recv().await {
            if sender
                .send(axum::extract::ws::Message::Text(event))
                .await
                .is_err()
            {
                break;
            }
        }
    });

    // Handle incoming messages from client
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(axum::extract::ws::Message::Text(text)) => {
                tracing::debug!("Received from client {}: {}", client_id, text);
                // Handle client messages if needed
            }
            Ok(axum::extract::ws::Message::Close(_)) => {
                tracing::info!("WebSocket closed: {}", client_id);
                break;
            }
            Err(e) => {
                tracing::error!("WebSocket error: {}", e);
                break;
            }
            _ => {}
        }
    }

    forward_task.abort();
    tracing::info!("WebSocket disconnected: {}", client_id);
}

// ============================================================================
// ComfyUI WebSocket Listener
// ============================================================================

/// Start listening to ComfyUI WebSocket for events
pub async fn start_comfyui_listener(
    comfyui: ComfyUIClient,
    event_tx: broadcast::Sender<String>,
    client_id: String,
) {
    let api_base = comfyui.public_base_url().to_string();

    loop {
        // Get current WebSocket URL (may have changed)
        let ws_url = format!("{}?clientId={}", comfyui.ws_url().await, client_id);

        match tokio_tungstenite::connect_async(&ws_url).await {
            Ok((ws_stream, _)) => {
                tracing::info!("Connected to ComfyUI WebSocket");

                let (_, mut read) = ws_stream.split();
                let mut current_prompt_id: Option<String> = None;

                while let Some(msg) = read.next().await {
                    match msg {
                        Ok(tokio_tungstenite::tungstenite::Message::Text(text)) => {
                            if let Ok(comfy_msg) = serde_json::from_str::<serde_json::Value>(&text)
                            {
                                let msg_type =
                                    comfy_msg.get("type").and_then(|v| v.as_str()).unwrap_or("");
                                if msg_type == "execution_start" {
                                    current_prompt_id = comfy_msg
                                        .get("data")
                                        .and_then(|v| v.get("prompt_id"))
                                        .and_then(|v| v.as_str())
                                        .map(|v| v.to_string());
                                }
                                // Log all non-monitor messages
                                if !msg_type.contains("monitor") && msg_type != "status" {
                                    tracing::info!("ComfyUI [{}]: {}", msg_type, text);
                                }
                                if let Some(frontend_msg) =
                                    convert_comfyui_message(&comfy_msg, &api_base)
                                {
                                    if let Ok(json) = serde_json::to_string(&frontend_msg) {
                                        let _ = event_tx.send(json);
                                    }
                                }
                            }
                        }
                        Ok(tokio_tungstenite::tungstenite::Message::Binary(data)) => {
                            // Handle binary preview images
                            if data.len() > 8 {
                                // First 4 bytes: type, next 4: format, rest: image data
                                let image_data = &data[8..];
                                let base64_image = BASE64.encode(image_data);
                                let prompt_id = current_prompt_id
                                    .as_deref()
                                    .unwrap_or("current")
                                    .to_string();
                                let preview_msg = FrontendMessage::Preview {
                                    prompt_id,
                                    image_data: format!("data:image/jpeg;base64,{}", base64_image),
                                };
                                if let Ok(json) = serde_json::to_string(&preview_msg) {
                                    let _ = event_tx.send(json);
                                }
                            }
                        }
                        Err(e) => {
                            tracing::debug!("ComfyUI WebSocket error: {}", e);
                            break;
                        }
                        _ => {}
                    }
                }
            }
            Err(e) => {
                tracing::debug!("ComfyUI WebSocket not available: {}", e);
            }
        }

        tracing::debug!("ComfyUI WebSocket disconnected, reconnecting in 5 seconds...");
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}

/// Convert ComfyUI message to frontend message
fn convert_comfyui_message(msg: &serde_json::Value, _api_base: &str) -> Option<FrontendMessage> {
    let msg_type = msg.get("type")?.as_str()?;
    let data = msg.get("data")?;

    match msg_type {
        "status" => {
            let queue_remaining = data
                .get("status")?
                .get("exec_info")?
                .get("queue_remaining")?
                .as_u64()? as u32;

            Some(FrontendMessage::QueueStatus {
                running: if queue_remaining > 0 { 1 } else { 0 },
                pending: queue_remaining.saturating_sub(1),
            })
        }
        "execution_start" => {
            let prompt_id = data.get("prompt_id")?.as_str()?.to_string();
            Some(FrontendMessage::Started { prompt_id })
        }
        "progress" => {
            let prompt_id = data.get("prompt_id")?.as_str()?.to_string();
            let node = data.get("node")?.as_str()?.to_string();
            let value = data.get("value")?.as_u64()? as u32;
            let max = data.get("max")?.as_u64()? as u32;
            let percentage = if max > 0 {
                (value as f32 / max as f32) * 100.0
            } else {
                0.0
            };

            Some(FrontendMessage::Progress {
                prompt_id,
                node,
                value,
                max,
                percentage,
            })
        }
        "executed" => {
            let prompt_id = data.get("prompt_id")?.as_str()?.to_string();
            let output = data.get("output")?;
            let images: Vec<ImageResult> = output
                .get("images")?
                .as_array()?
                .iter()
                .filter_map(|img| {
                    let img_type = img.get("type")?.as_str()?.to_string();
                    if img_type != "output" {
                        return None;
                    }
                    let filename = img.get("filename")?.as_str()?.to_string();
                    let subfolder = img.get("subfolder")?.as_str()?.to_string();
                    Some(ImageResult {
                        filename,
                        subfolder,
                        image_type: img_type,
                    })
                })
                .collect();

            if !images.is_empty() {
                Some(FrontendMessage::Completed { prompt_id, images })
            } else {
                None
            }
        }
        "execution_error" => {
            let prompt_id = data.get("prompt_id")?.as_str()?.to_string();
            let exception_message = data
                .get("exception_message")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error")
                .to_string();

            Some(FrontendMessage::Error {
                prompt_id: Some(prompt_id),
                message: exception_message,
            })
        }
        _ => None,
    }
}
