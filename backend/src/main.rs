mod api;
mod comfyui;
mod config;
mod error;
mod models;
mod services;

use axum::http::HeaderValue;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tower_http::cors::{AllowOrigin, Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::api::{create_router, start_comfyui_listener, AppState};
use crate::comfyui::ComfyUIClient;
use crate::config::Config;
use crate::services::prompt_optimizer::PromptOptimizer;

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=info,axum::rejection=trace".into()),
        )
        .init();

    // Load configuration
    let config = Arc::new(Config::from_env());

    // Create ComfyUI client
    let comfyui = ComfyUIClient::new(config.clone());

    // Create event broadcast channel
    let (event_tx, _) = broadcast::channel::<String>(100);

    // Create application state
    let comfyui_client_id = uuid::Uuid::new_v4().to_string();

    let state = AppState {
        comfyui: comfyui.clone(),
        event_tx: event_tx.clone(),
        comfyui_client_id: comfyui_client_id.clone(),
        prompt_optimizer: PromptOptimizer::new(),
        config: config.clone(),
    };

    // Configure CORS (local-only by default)
    let origin_list: Vec<HeaderValue> = config
        .cors_origins
        .iter()
        .filter_map(|origin| origin.parse().ok())
        .collect();

    let cors = if origin_list.is_empty() {
        CorsLayer::new()
    } else {
        CorsLayer::new().allow_origin(AllowOrigin::list(origin_list))
    }
    .allow_methods(Any)
    .allow_headers(Any);

    // Create router with middleware
    let app = create_router(state)
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    // Start ComfyUI WebSocket listener in background
    let comfyui_for_listener = comfyui.clone();
    let event_tx_for_listener = event_tx.clone();
    let client_id_for_listener = comfyui_client_id.clone();
    tokio::spawn(async move {
        start_comfyui_listener(
            comfyui_for_listener,
            event_tx_for_listener,
            client_id_for_listener,
        )
        .await;
    });

    // Start server
    let addr = config.server_addr();
    let port = config.port;
    let listener = TcpListener::bind(&addr)
        .await
        .expect("Failed to bind listener");

    let url = format!("http://localhost:{}", port);
    tracing::info!("Server running at {}", url);

    // Open browser only in release builds (skip during dev)
    if cfg!(not(debug_assertions)) {
        #[cfg(target_os = "windows")]
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", &url])
            .spawn();
        #[cfg(target_os = "macos")]
        let _ = std::process::Command::new("open").arg(&url).spawn();
        #[cfg(target_os = "linux")]
        let _ = std::process::Command::new("xdg-open").arg(&url).spawn();
    }

    axum::serve(listener, app).await.expect("Server crashed");
}
