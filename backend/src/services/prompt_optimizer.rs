use crate::error::AppResult;
use crate::models::{OptimizePromptRequest, OptimizePromptResponse};

/// Minimal prompt optimizer stub.
#[derive(Clone, Default)]
pub struct PromptOptimizer;

impl PromptOptimizer {
    pub fn new() -> Self {
        Self
    }

    pub async fn optimize(
        &self,
        request: OptimizePromptRequest,
    ) -> AppResult<OptimizePromptResponse> {
        let mut optimized_prompt = request.prompt;
        let tags = request.tags.trim();
        if !tags.is_empty() {
            if !optimized_prompt.is_empty() {
                optimized_prompt.push('\n');
            }
            optimized_prompt.push_str(tags);
        }

        Ok(OptimizePromptResponse {
            optimized_prompt,
            candidates: None,
        })
    }
}
