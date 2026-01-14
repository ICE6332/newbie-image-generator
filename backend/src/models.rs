use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Request Models (from frontend)
// ============================================================================

/// Image generation request from the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateRequest {
    /// Positive prompt text
    pub prompt: String,
    /// Optional system prompt to prefix before user prompt
    pub system_prompt: Option<String>,
    /// Negative prompt text
    #[serde(default)]
    pub negative_prompt: String,
    /// Image width
    #[serde(default = "default_width")]
    pub width: u32,
    /// Image height
    #[serde(default = "default_height")]
    pub height: u32,
    /// Number of sampling steps
    #[serde(default = "default_steps")]
    pub steps: u32,
    /// CFG scale
    #[serde(default = "default_cfg")]
    pub cfg: f32,
    /// Random seed (-1 for random)
    #[serde(default = "default_seed")]
    pub seed: i64,
    /// Sampler name
    #[serde(default = "default_sampler")]
    pub sampler_name: String,
    /// Scheduler name
    #[serde(default = "default_scheduler")]
    pub scheduler: String,
    /// Denoise strength
    #[serde(default = "default_denoise")]
    pub denoise: f32,
    /// Batch size
    #[serde(default = "default_batch_size")]
    pub batch_size: u32,
    /// LoRA configurations
    #[serde(default)]
    pub loras: Vec<LoraConfig>,
    /// HiFix enabled
    #[serde(default)]
    pub hifix_enabled: bool,
    /// HiFix steps
    #[serde(default = "default_hifix_steps")]
    pub hifix_steps: u32,
    /// HiFix CFG
    #[serde(default = "default_hifix_cfg")]
    pub hifix_cfg: f32,
    /// HiFix denoise
    #[serde(default = "default_hifix_denoise")]
    pub hifix_denoise: f32,
    /// HiFix sampler
    #[serde(default = "default_hifix_sampler")]
    pub hifix_sampler: String,
    /// HiFix scheduler
    #[serde(default = "default_hifix_scheduler")]
    pub hifix_scheduler: String,
    /// HiFix scale
    #[serde(default = "default_hifix_scale")]
    pub hifix_scale: f32,
    /// HiFix upscale method
    #[serde(default = "default_hifix_upscale_method")]
    pub hifix_upscale_method: String,
}

/// LoRA configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoraConfig {
    /// LoRA file name
    pub name: String,
    /// LoRA strength (-4.0 to 4.0)
    #[serde(default = "default_lora_strength")]
    pub strength: f32,
}

fn default_lora_strength() -> f32 {
    1.0
}

/// Prompt optimization request (Gemma + Jina CLIP rerank)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizePromptRequest {
    /// User handwritten prompt (any language)
    pub prompt: String,
    /// Optional additional tags (raw text)
    #[serde(default)]
    pub tags: String,
    /// Optional reference image as a data URL (data:image/...;base64,...) or bare base64
    #[serde(default)]
    pub reference_image: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizePromptResponse {
    /// Final JSON string (includes XML in image.tags) that should replace the user's prompt
    pub optimized_prompt: String,
    /// Optional extra candidates for UI preview
    #[serde(skip_serializing_if = "Option::is_none")]
    pub candidates: Option<Vec<String>>,
}

fn default_width() -> u32 {
    1024
}
fn default_height() -> u32 {
    1536
}
fn default_steps() -> u32 {
    28
}
fn default_cfg() -> f32 {
    4.5
}
fn default_seed() -> i64 {
    -1
}
fn default_sampler() -> String {
    "res_multistep".to_string()
}
fn default_scheduler() -> String {
    "linear_quadratic".to_string()
}
fn default_denoise() -> f32 {
    1.0
}
fn default_batch_size() -> u32 {
    1
}
fn default_hifix_steps() -> u32 {
    20
}
fn default_hifix_cfg() -> f32 {
    4.0
}
fn default_hifix_denoise() -> f32 {
    0.5
}
fn default_hifix_sampler() -> String {
    "res_multistep".to_string()
}
fn default_hifix_scheduler() -> String {
    "linear_quadratic".to_string()
}
fn default_hifix_scale() -> f32 {
    1.5
}
fn default_hifix_upscale_method() -> String {
    "nearest-exact".to_string()
}

// ============================================================================
// Response Models (to frontend)
// ============================================================================

/// Queue response after submitting a prompt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueResponse {
    pub prompt_id: String,
    pub number: u32,
}

/// Generation progress update
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressUpdate {
    pub prompt_id: String,
    pub node: Option<String>,
    pub value: u32,
    pub max: u32,
    #[serde(rename = "type")]
    pub update_type: String,
}

/// Generated image result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedImage {
    pub filename: String,
    pub subfolder: String,
    #[serde(rename = "type")]
    pub image_type: String,
}

/// Complete generation result
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationResult {
    pub prompt_id: String,
    pub images: Vec<GeneratedImage>,
    pub status: String,
}

// ============================================================================
// ComfyUI API Models
// ============================================================================

/// ComfyUI queue prompt request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComfyUIPromptRequest {
    pub prompt: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
}

/// ComfyUI queue prompt response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComfyUIPromptResponse {
    pub prompt_id: String,
    pub number: u32,
    pub node_errors: HashMap<String, serde_json::Value>,
}

/// ComfyUI history response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComfyUIHistoryResponse {
    #[serde(flatten)]
    pub prompts: HashMap<String, PromptHistory>,
}

/// Single prompt history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptHistory {
    pub prompt: serde_json::Value,
    pub outputs: HashMap<String, NodeOutput>,
    #[serde(default)]
    pub status: PromptStatus,
}

/// Prompt execution status
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PromptStatus {
    pub status_str: Option<String>,
    pub completed: Option<bool>,
    pub messages: Option<Vec<serde_json::Value>>,
}

/// Output from a node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeOutput {
    #[serde(default)]
    pub images: Vec<GeneratedImage>,
}

/// ComfyUI system stats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStats {
    pub system: SystemInfo,
    pub devices: Vec<DeviceInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub os: String,
    pub python_version: String,
    pub embedded_python: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub name: String,
    #[serde(rename = "type")]
    pub device_type: String,
    pub index: i32,
    pub vram_total: u64,
    pub vram_free: u64,
    pub torch_vram_total: u64,
    pub torch_vram_free: u64,
}

/// ComfyUI queue status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueStatus {
    pub queue_running: Vec<serde_json::Value>,
    pub queue_pending: Vec<serde_json::Value>,
}

// ============================================================================
// WebSocket Message Models
// ============================================================================

/// WebSocket message types from ComfyUI
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ComfyUIMessage {
    #[serde(rename = "status")]
    Status { status: StatusData },
    #[serde(rename = "execution_start")]
    ExecutionStart { prompt_id: String },
    #[serde(rename = "execution_cached")]
    ExecutionCached {
        nodes: Vec<String>,
        prompt_id: String,
    },
    #[serde(rename = "executing")]
    Executing {
        node: Option<String>,
        prompt_id: String,
    },
    #[serde(rename = "progress")]
    Progress {
        value: u32,
        max: u32,
        prompt_id: String,
        node: String,
    },
    #[serde(rename = "executed")]
    Executed {
        node: String,
        output: NodeOutput,
        prompt_id: String,
    },
    #[serde(rename = "execution_error")]
    ExecutionError {
        prompt_id: String,
        node_id: String,
        node_type: String,
        exception_message: String,
    },
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusData {
    pub exec_info: ExecInfo,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecInfo {
    pub queue_remaining: u32,
}

// ============================================================================
// Frontend WebSocket Messages
// ============================================================================

/// Messages sent to the frontend via WebSocket
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum FrontendMessage {
    #[serde(rename = "connected")]
    Connected { client_id: String },
    #[serde(rename = "queued")]
    Queued {
        prompt_id: String,
        queue_position: u32,
    },
    #[serde(rename = "started")]
    Started { prompt_id: String },
    #[serde(rename = "progress")]
    Progress {
        prompt_id: String,
        node: String,
        value: u32,
        max: u32,
        percentage: f32,
    },
    #[serde(rename = "preview")]
    Preview {
        prompt_id: String,
        image_data: String,
    },
    #[serde(rename = "completed")]
    Completed {
        prompt_id: String,
        images: Vec<ImageResult>,
    },
    #[serde(rename = "error")]
    Error {
        prompt_id: Option<String>,
        message: String,
    },
    #[serde(rename = "queue_status")]
    QueueStatus { running: u32, pending: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageResult {
    pub filename: String,
    pub subfolder: String,
    #[serde(rename = "type")]
    pub image_type: String,
}

// ============================================================================
// Available Models
// ============================================================================

#[derive(Debug, Clone, Default)]
pub struct AvailableModels {
    pub unet: Vec<String>,
    pub clip: Vec<String>,
    pub vae: Vec<String>,
}

/// Find a model file by keywords (tries each in order, case-insensitive)
pub fn find_model(models: &[String], keywords: &[&str]) -> Option<String> {
    for keyword in keywords {
        let keyword_lower = keyword.to_lowercase();
        if let Some(m) = models
            .iter()
            .find(|m| m.to_lowercase().contains(&keyword_lower))
        {
            return Some(m.clone());
        }
    }
    None
}
