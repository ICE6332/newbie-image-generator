use crate::config::Config;
use crate::error::{AppError, AppResult};
use crate::models::*;
use reqwest::{Client, Url};
use serde_json::{json, Value};
use std::sync::Arc;

pub use crate::models::{find_model, AvailableModels};

/// ComfyUI client for interacting with the ComfyUI server
#[derive(Clone)]
pub struct ComfyUIClient {
    client: Client,
    config: Arc<Config>,
}

impl ComfyUIClient {
    /// Create a new ComfyUI client
    pub fn new(config: Arc<Config>) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .build()
            .expect("Failed to create HTTP client");

        Self { client, config }
    }

    /// Get the base URL for ComfyUI
    pub async fn base_url(&self) -> String {
        self.config.comfyui.read().await.url.clone()
    }

    /// Get the WebSocket URL for ComfyUI
    pub async fn ws_url(&self) -> String {
        self.config.comfyui.read().await.ws_url.clone()
    }

    /// Get the public base URL for this backend
    pub fn public_base_url(&self) -> &str {
        &self.config.public_base_url
    }

    /// Update ComfyUI URL
    pub async fn set_url(&self, url: &str) {
        let mut comfyui = self.config.comfyui.write().await;
        *comfyui = crate::config::ComfyUIConfig::new(url);
    }

    /// Get current ComfyUI URL
    pub async fn get_url(&self) -> String {
        self.config.comfyui.read().await.url.clone()
    }

    /// Check if ComfyUI is reachable
    pub async fn health_check(&self) -> AppResult<bool> {
        let base_url = self.base_url().await;
        let url = format!("{}/system_stats", base_url);
        match self.client.get(&url).send().await {
            Ok(resp) => Ok(resp.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    /// Get system stats from ComfyUI
    pub async fn get_system_stats(&self) -> AppResult<SystemStats> {
        let url = format!("{}/system_stats", self.base_url().await);
        let resp = self.client.get(&url).send().await?;

        if !resp.status().is_success() {
            return Err(AppError::ComfyUIApi(format!(
                "Failed to get system stats: {}",
                resp.status()
            )));
        }

        resp.json()
            .await
            .map_err(|e| AppError::ComfyUIApi(e.to_string()))
    }

    /// Get queue status
    pub async fn get_queue(&self) -> AppResult<QueueStatus> {
        let url = format!("{}/queue", self.base_url().await);
        let resp = self.client.get(&url).send().await?;

        if !resp.status().is_success() {
            return Err(AppError::ComfyUIApi(format!(
                "Failed to get queue: {}",
                resp.status()
            )));
        }

        resp.json()
            .await
            .map_err(|e| AppError::ComfyUIApi(e.to_string()))
    }

    /// Queue a prompt for execution
    pub async fn queue_prompt(
        &self,
        workflow: Value,
        client_id: Option<String>,
    ) -> AppResult<ComfyUIPromptResponse> {
        let url = format!("{}/prompt", self.base_url().await);

        let request = ComfyUIPromptRequest {
            prompt: workflow,
            client_id,
        };

        let resp = self.client.post(&url).json(&request).send().await?;

        if !resp.status().is_success() {
            let error_text = resp.text().await.unwrap_or_default();
            return Err(AppError::ComfyUIApi(format!(
                "Failed to queue prompt: {}",
                error_text
            )));
        }

        resp.json()
            .await
            .map_err(|e| AppError::ComfyUIApi(e.to_string()))
    }

    /// Get history for a specific prompt
    pub async fn get_history(&self, prompt_id: &str) -> AppResult<Option<PromptHistory>> {
        let prompt_url = format!("{}/history/{}", self.base_url().await, prompt_id);
        let resp = self.client.get(&prompt_url).send().await?;

        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return self.get_history_from_all(prompt_id).await;
        }

        if !resp.status().is_success() {
            return Err(AppError::ComfyUIApi(format!(
                "Failed to get history: {}",
                resp.status()
            )));
        }

        let text = resp
            .text()
            .await
            .map_err(|e| AppError::ComfyUIApi(e.to_string()))?;

        if text.trim().is_empty() || text.trim() == "{}" {
            return self.get_history_from_all(prompt_id).await;
        }

        if let Ok(history) = serde_json::from_str::<ComfyUIHistoryResponse>(&text) {
            if let Some(prompt) = history.prompts.get(prompt_id) {
                return Ok(Some(prompt.clone()));
            }
        }

        if let Ok(prompt) = serde_json::from_str::<PromptHistory>(&text) {
            return Ok(Some(prompt));
        }

        Err(AppError::ComfyUIApi(
            "Failed to parse history response".to_string(),
        ))
    }

    async fn get_history_from_all(&self, prompt_id: &str) -> AppResult<Option<PromptHistory>> {
        let url = format!("{}/history", self.base_url().await);
        let resp = self.client.get(&url).send().await?;

        if !resp.status().is_success() {
            return Err(AppError::ComfyUIApi(format!(
                "Failed to get history: {}",
                resp.status()
            )));
        }

        let text = resp
            .text()
            .await
            .map_err(|e| AppError::ComfyUIApi(e.to_string()))?;

        if text.trim().is_empty() || text.trim() == "{}" {
            return Ok(None);
        }

        let history: ComfyUIHistoryResponse = serde_json::from_str(&text)
            .map_err(|e| AppError::ComfyUIApi(format!("Failed to parse history: {}", e)))?;

        Ok(history.prompts.get(prompt_id).cloned())
    }

    /// Get an image from ComfyUI
    pub async fn get_image(
        &self,
        filename: &str,
        subfolder: &str,
        image_type: &str,
    ) -> AppResult<Vec<u8>> {
        let mut url = Url::parse(&format!("{}/view", self.base_url().await))
            .map_err(|e| AppError::Internal(e.to_string()))?;
        url.query_pairs_mut()
            .append_pair("filename", filename)
            .append_pair("subfolder", subfolder)
            .append_pair("type", image_type);

        let resp = self.client.get(url).send().await?;

        if !resp.status().is_success() {
            return Err(AppError::ComfyUIApi(format!(
                "Failed to get image: {}",
                resp.status()
            )));
        }

        resp.bytes()
            .await
            .map(|b| b.to_vec())
            .map_err(|e| AppError::ComfyUIApi(e.to_string()))
    }

    /// Cancel the current execution
    pub async fn interrupt(&self) -> AppResult<()> {
        let url = format!("{}/interrupt", self.base_url().await);
        let resp = self.client.post(&url).send().await?;

        if !resp.status().is_success() {
            return Err(AppError::ComfyUIApi(format!(
                "Failed to interrupt: {}",
                resp.status()
            )));
        }

        Ok(())
    }

    /// Clear the queue
    pub async fn clear_queue(&self) -> AppResult<()> {
        let url = format!("{}/queue", self.base_url().await);
        let resp = self
            .client
            .post(&url)
            .json(&json!({"clear": true}))
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(AppError::ComfyUIApi(format!(
                "Failed to clear queue: {}",
                resp.status()
            )));
        }

        Ok(())
    }

    /// Get available models from ComfyUI
    pub async fn get_available_models(&self) -> AppResult<AvailableModels> {
        let url = format!("{}/object_info", self.base_url().await);
        let resp = self.client.get(&url).send().await?;

        if !resp.status().is_success() {
            return Err(AppError::ComfyUIApi(format!(
                "Failed to get object info: {}",
                resp.status()
            )));
        }

        let info: Value = resp
            .json()
            .await
            .map_err(|e| AppError::ComfyUIApi(e.to_string()))?;

        let mut models = AvailableModels::default();

        // Get UNET models
        if let Some(unet_list) = info
            .get("UNETLoader")
            .and_then(|v| v.get("input"))
            .and_then(|v| v.get("required"))
            .and_then(|v| v.get("unet_name"))
            .and_then(|v| v.get(0))
            .and_then(|v| v.as_array())
        {
            models.unet = unet_list
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect();
        }

        // Get CLIP models
        if let Some(clip_list) = info
            .get("DualCLIPLoader")
            .and_then(|v| v.get("input"))
            .and_then(|v| v.get("required"))
            .and_then(|v| v.get("clip_name1"))
            .and_then(|v| v.get(0))
            .and_then(|v| v.as_array())
        {
            models.clip = clip_list
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect();
        }

        // Get VAE models
        if let Some(vae_list) = info
            .get("VAELoader")
            .and_then(|v| v.get("input"))
            .and_then(|v| v.get("required"))
            .and_then(|v| v.get("vae_name"))
            .and_then(|v| v.get(0))
            .and_then(|v| v.as_array())
        {
            models.vae = vae_list
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect();
        }

        Ok(models)
    }

    /// Build workflow from generation request based on newbie-api.json template
    pub fn build_workflow(&self, request: &GenerateRequest, models: &AvailableModels) -> Value {
        let seed = if request.seed < 0 {
            rand_seed()
        } else {
            request.seed as u64
        };

        // Find matching model files (case-insensitive, try keywords in order)
        let unet_name = find_model(&models.unet, &["newbie"])
            .unwrap_or_else(|| "newbie01.safetensors".to_string());
        let clip_name1 = find_model(
            &models.clip,
            &["gemma_3_4b", "gemma3_4b", "gemma_3", "gemma3", "gemma"],
        )
        .unwrap_or_else(|| "gemma3-4b-it.safetensors".to_string());
        let clip_name2 = find_model(&models.clip, &["jina"])
            .unwrap_or_else(|| "jina-clip-v2.safetensors".to_string());
        let vae_name = find_model(&models.vae, &["newbie", "diffusion_pytorch"])
            .unwrap_or_else(|| "newbie-image.safetensors".to_string());

        // Build the final prompt on the backend (frontend provides system_prompt + user prompt).
        let positive_prompt = compose_positive_prompt(request);
        let negative_prompt = if request.negative_prompt.is_empty() {
            String::new()
        } else {
            format!("<danbooru_tags>{}</danbooru_tags>", request.negative_prompt)
        };

        // VAEDecode input: from HiFix KSampler if enabled, otherwise from first KSampler
        let vae_decode_input = if request.hifix_enabled {
            json!(["101", 0])
        } else {
            json!(["3", 0])
        };

        let mut workflow = json!({
            "3": {
                "inputs": {
                    "seed": seed,
                    "steps": request.steps,
                    "cfg": request.cfg,
                    "sampler_name": request.sampler_name,
                    "scheduler": request.scheduler,
                    "denoise": request.denoise,
                    "model": ["51", 0],
                    "positive": ["61", 0],
                    "negative": ["59", 0],
                    "latent_image": ["9", 0]
                },
                "class_type": "KSampler",
                "_meta": {"title": "K采样器"}
            },
            "4": {
                "inputs": {
                    "samples": vae_decode_input,
                    "vae": ["5", 0]
                },
                "class_type": "VAEDecode",
                "_meta": {"title": "VAE解码"}
            },
            "5": {
                "inputs": {
                    "vae_name": vae_name
                },
                "class_type": "VAELoader",
                "_meta": {"title": "VAE加载器"}
            },
            "9": {
                "inputs": {
                    "width": request.width,
                    "height": request.height,
                    "batch_size": request.batch_size
                },
                "class_type": "EmptySD3LatentImage",
                "_meta": {"title": "空Latent_SD3"}
            },
            "39": {
                "inputs": {
                    "filename_prefix": "ComfyUI",
                    "images": ["4", 0]
                },
                "class_type": "SaveImage",
                "_meta": {"title": "保存图像"}
            },
            "40": {
                "inputs": {
                    "images": ["4", 0]
                },
                "class_type": "PreviewImage",
                "_meta": {"title": "预览图像"}
            },
            "54": {
                "inputs": {
                    "unet_name": unet_name,
                    "weight_dtype": "default"
                },
                "class_type": "UNETLoader",
                "_meta": {"title": "UNET加载器"}
            },
            "58": {
                "inputs": {
                    "clip_name1": clip_name1,
                    "clip_name2": clip_name2,
                    "type": "newbie",
                    "device": "default"
                },
                "class_type": "DualCLIPLoader",
                "_meta": {"title": "双CLIP加载器"}
            }
        });

        // Build LoRA chain and determine final model/clip outputs
        let mut model_output = json!(["54", 0]); // Start from UNETLoader
        let mut clip_output = json!(["58", 0]);  // Start from DualCLIPLoader

        for (i, lora) in request.loras.iter().enumerate() {
            let node_id = format!("{}", 200 + i);
            workflow[&node_id] = json!({
                "inputs": {
                    "lora_name": lora.name,
                    "strength": lora.strength,
                    "model": model_output,
                    "clip": clip_output
                },
                "class_type": "NewBieLoraLoader",
                "_meta": {"title": format!("LoRA加载器 {}", i + 1)}
            });
            model_output = json!([node_id, 0]);
            clip_output = json!([node_id, 1]);
        }

        // RescaleCFG uses final model output
        workflow["51"] = json!({
            "inputs": {
                "multiplier": 0.9,
                "model": model_output
            },
            "class_type": "RescaleCFG",
            "_meta": {"title": "缩放CFG"}
        });

        // CLIPTextEncode nodes use final clip output
        workflow["59"] = json!({
            "inputs": {
                "text": negative_prompt,
                "clip": clip_output
            },
            "class_type": "CLIPTextEncode",
            "_meta": {"title": "CLIP文本编码器"}
        });
        workflow["61"] = json!({
            "inputs": {
                "text": positive_prompt,
                "clip": clip_output
            },
            "class_type": "CLIPTextEncode",
            "_meta": {"title": "CLIP文本编码器"}
        });

        // Add HiFix nodes if enabled
        if request.hifix_enabled {
            let hifix_width = (request.width as f32 * request.hifix_scale) as u32;
            let hifix_height = (request.height as f32 * request.hifix_scale) as u32;

            workflow["100"] = json!({
                "inputs": {
                    "upscale_method": request.hifix_upscale_method,
                    "width": hifix_width,
                    "height": hifix_height,
                    "crop": "disabled",
                    "samples": ["3", 0]
                },
                "class_type": "LatentUpscale",
                "_meta": {"title": "Latent放大"}
            });

            workflow["101"] = json!({
                "inputs": {
                    "seed": seed,
                    "steps": request.hifix_steps,
                    "cfg": request.hifix_cfg,
                    "sampler_name": request.hifix_sampler,
                    "scheduler": request.hifix_scheduler,
                    "denoise": request.hifix_denoise,
                    "model": ["51", 0],
                    "positive": ["61", 0],
                    "negative": ["59", 0],
                    "latent_image": ["100", 0]
                },
                "class_type": "KSampler",
                "_meta": {"title": "HiFix采样器"}
            });
        }

        workflow
    }
}

fn compose_positive_prompt(request: &GenerateRequest) -> String {
    let user_prompt = normalize_prompt(&request.prompt);
    let system_prompt = request.system_prompt.as_deref().unwrap_or("").trim();

    if system_prompt.is_empty() {
        format!("<Prompt Start>,{}", user_prompt)
    } else {
        format!("{}\n<Prompt Start>,{}", system_prompt, user_prompt)
    }
}

fn normalize_prompt(raw: &str) -> String {
    let mut s = raw;
    if let Some(stripped) = s.strip_prefix('\u{feff}') {
        s = stripped;
    }

    let s = s.replace("\r\n", "\n").replace('\r', "\n");
    let mut s = s.trim_start();

    // If the user already pasted a Prompt Start header, remove it to avoid duplication.
    let prompt_start = "<prompt start>";
    if s.len() >= prompt_start.len() && s[..prompt_start.len()].eq_ignore_ascii_case(prompt_start) {
        s = s[prompt_start.len()..]
            .trim_start_matches(|c: char| c.is_whitespace() || c == ',' || c == ':');
    }

    let s = s.trim();
    if s.is_empty() {
        return String::new();
    }

    let mut lines = Vec::new();
    for line in s.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let mut out = String::with_capacity(trimmed.len());
        let mut prev_space = false;
        for ch in trimmed.chars() {
            if ch.is_whitespace() {
                if !prev_space {
                    out.push(' ');
                    prev_space = true;
                }
            } else {
                out.push(ch);
                prev_space = false;
            }
        }

        if !out.is_empty() {
            lines.push(out);
        }
    }

    lines.join("\n")
}

/// Generate a random seed
fn rand_seed() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    duration.as_nanos() as u64 % 1_000_000_000_000_000
}
