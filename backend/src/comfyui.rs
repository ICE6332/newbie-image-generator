use crate::config::Config;
use crate::error::{AppError, AppResult};
use crate::models::*;
use reqwest::Client;
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
        let url = format!(
            "{}/view?filename={}&subfolder={}&type={}",
            self.base_url().await,
            filename,
            subfolder,
            image_type
        );

        let resp = self.client.get(&url).send().await?;

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

        // Build the prompt prefix for newbie model
        let positive_prompt = format!(
            "You are an assistant designed to generate high-quality anime images with the highest degree of image-text alignment based on xml format textual prompts. <Prompt Start>\n{}",
            request.prompt
        );

        // Default negative prompt if empty
        let negative_prompt = if request.negative_prompt.is_empty() {
            "<danbooru_tags>low_score_rate, worst quality, low quality, bad quality, lowres, low res, pixelated, blurry, blurred, compression artifacts, jpeg artifacts, bad anatomy, worst hands, deformed hands, deformed fingers, deformed feet, deformed toes, extra limbs, extra arms, extra legs, extra fingers, extra digits, extra digit, fused fingers, missing limbs, missing arms, missing fingers, missing toes, wrong hands, ugly hands, ugly fingers, twisted hands, flexible deformity, conjoined, disembodied, text, watermark, signature, logo, ugly, worst, very displeasing, displeasing, error, doesnotexist, unfinished, poorly drawn face, poorly drawn hands, poorly drawn feet, artistic error, bad proportions, bad perspective, out of frame, ai-generated, ai-assisted, stable diffusion, overly saturated, overly vivid, cross-eye, expressionless, scan, sketch, monochrome, simple background, abstract, sequence, lineup, 2koma, 4koma, microsoft paint \\(medium\\), artifacts, adversarial noise, has bad revision, resized, image sample,low_aesthetic</danbooru_tags>".to_string()
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
            "51": {
                "inputs": {
                    "multiplier": 0.9,
                    "model": ["54", 0]
                },
                "class_type": "RescaleCFG",
                "_meta": {"title": "缩放CFG"}
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
            },
            "59": {
                "inputs": {
                    "text": negative_prompt,
                    "clip": ["58", 0]
                },
                "class_type": "CLIPTextEncode",
                "_meta": {"title": "CLIP文本编码器"}
            },
            "61": {
                "inputs": {
                    "text": positive_prompt,
                    "clip": ["58", 0]
                },
                "class_type": "CLIPTextEncode",
                "_meta": {"title": "CLIP文本编码器"}
            }
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

/// Generate a random seed
fn rand_seed() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    duration.as_nanos() as u64 % 1_000_000_000_000_000
}
