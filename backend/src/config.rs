use std::env;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Dynamic ComfyUI configuration that can be updated at runtime
#[derive(Debug, Clone)]
pub struct ComfyUIConfig {
    pub url: String,
    pub ws_url: String,
}

impl ComfyUIConfig {
    pub fn new(url: &str) -> Self {
        let url = url.trim_end_matches('/').to_string();
        let ws_url = url
            .replace("http://", "ws://")
            .replace("https://", "wss://")
            + "/ws";
        Self { url, ws_url }
    }
}

/// Application configuration loaded from environment variables
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Config {
    /// Server host address
    pub host: String,
    /// Server port
    pub port: u16,
    /// ComfyUI configuration (dynamic)
    pub comfyui: Arc<RwLock<ComfyUIConfig>>,
    /// Public base URL for this backend (used in image URLs)
    pub public_base_url: String,
    /// Allowed CORS origins
    pub cors_origins: Vec<String>,
    /// Allow remote ComfyUI URLs (SSRF risk if enabled)
    pub allow_remote_comfyui: bool,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port = env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
            .expect("PORT must be a valid number");

        let comfyui_host = env::var("COMFYUI_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let comfyui_port = env::var("COMFYUI_PORT").unwrap_or_else(|_| "8188".to_string());
        let comfyui_url = format!("http://{}:{}", comfyui_host, comfyui_port);

        let public_base_url = env::var("PUBLIC_BASE_URL")
            .unwrap_or_else(|_| format!("http://localhost:{}", port))
            .trim_end_matches('/')
            .to_string();

        let cors_origins = env::var("CORS_ORIGINS")
            .unwrap_or_else(|_| "http://localhost:3001,http://127.0.0.1:3001".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        let allow_remote_comfyui = env::var("ALLOW_REMOTE_COMFYUI")
            .unwrap_or_else(|_| "false".to_string())
            .to_lowercase();

        Self {
            host,
            port,
            comfyui: Arc::new(RwLock::new(ComfyUIConfig::new(&comfyui_url))),
            public_base_url,
            cors_origins,
            allow_remote_comfyui: matches!(allow_remote_comfyui.as_str(), "1" | "true" | "yes"),
        }
    }

    /// Get the server address string
    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
