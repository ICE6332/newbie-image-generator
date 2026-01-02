// API Types matching backend models

export interface GenerateRequest {
  prompt: string;
  negative_prompt?: string;
  width?: number;
  height?: number;
  steps?: number;
  cfg?: number;
  seed?: number;
  sampler_name?: string;
  scheduler?: string;
  denoise?: number;
  batch_size?: number;
  hifix_enabled?: boolean;
  hifix_steps?: number;
  hifix_cfg?: number;
  hifix_denoise?: number;
  hifix_sampler?: string;
  hifix_scheduler?: string;
  hifix_scale?: number;
  hifix_upscale_method?: string;
}

export interface QueueResponse {
  prompt_id: string;
  number: number;
}

export interface ImageResult {
  filename: string;
  subfolder: string;
  type: string;
}

export interface HistoryResponse {
  prompt_id: string;
  status: string;
  completed: boolean;
  images: ImageResult[];
}

export interface QueueStatus {
  running: number;
  pending: number;
  running_prompts: unknown[];
  pending_prompts: unknown[];
}

export interface SystemStatus {
  comfyui: {
    connected: boolean;
    system: {
      os: string;
      python_version: string;
      embedded_python: boolean;
    };
    devices: Array<{
      name: string;
      type: string;
      index: number;
      vram_total: number;
      vram_free: number;
      torch_vram_total: number;
      torch_vram_free: number;
    }>;
  };
  queue: {
    running: number;
    pending: number;
  };
}

export interface HealthResponse {
  status: string;
  comfyui: boolean;
}

// WebSocket message types
export type WSMessageType =
  | "connected"
  | "queued"
  | "started"
  | "progress"
  | "preview"
  | "completed"
  | "error"
  | "queue_status";

export interface WSConnectedMessage {
  type: "connected";
  client_id: string;
}

export interface WSQueuedMessage {
  type: "queued";
  prompt_id: string;
  queue_position: number;
}

export interface WSStartedMessage {
  type: "started";
  prompt_id: string;
}

export interface WSProgressMessage {
  type: "progress";
  prompt_id: string;
  node: string;
  value: number;
  max: number;
  percentage: number;
}

export interface WSPreviewMessage {
  type: "preview";
  prompt_id: string;
  image_data: string;
}

export interface WSCompletedMessage {
  type: "completed";
  prompt_id: string;
  images: ImageResult[];
}

export interface WSErrorMessage {
  type: "error";
  prompt_id: string | null;
  message: string;
}

export interface WSQueueStatusMessage {
  type: "queue_status";
  running: number;
  pending: number;
}

export type WSMessage =
  | WSConnectedMessage
  | WSQueuedMessage
  | WSStartedMessage
  | WSProgressMessage
  | WSPreviewMessage
  | WSCompletedMessage
  | WSErrorMessage
  | WSQueueStatusMessage;
