import type {
  GenerateRequest,
  QueueResponse,
  HistoryResponse,
  QueueStatus,
  SystemStatus,
  HealthResponse,
} from "./types";
import { resolveApiBase } from "./config";

const API_BASE = resolveApiBase();

class ApiError extends Error {
  status: number;

  constructor(status: number, message: string) {
    super(message);
    this.status = status;
    this.name = "ApiError";
  }
}

async function request<T>(endpoint: string, options?: RequestInit): Promise<T> {
  const response = await fetch(`${API_BASE}${endpoint}`, {
    headers: {
      "Content-Type": "application/json",
      ...options?.headers,
    },
    ...options,
  });

  if (!response.ok) {
    const error = await response
      .json()
      .catch(() => ({ error: "Unknown error" }));
    throw new ApiError(response.status, error.error || "Request failed");
  }

  return response.json();
}

export const api = {
  // Health check (served under /api/health to work with dev proxy)
  async health(): Promise<HealthResponse> {
    return request("/health");
  },

  // System status
  async status(): Promise<SystemStatus> {
    return request("/status");
  },

  // Generate image
  async generate(params: GenerateRequest): Promise<QueueResponse> {
    return request("/generate", {
      method: "POST",
      body: JSON.stringify(params),
    });
  },

  // Get queue status
  async queue(): Promise<QueueStatus> {
    return request("/queue");
  },

  // Get history for a prompt
  async history(promptId: string): Promise<HistoryResponse> {
    return request(`/history/${promptId}`);
  },

  // Get image URL
  getImageUrl(filename: string, subfolder = "", type = "output"): string {
    return `${API_BASE}/images/${filename}?subfolder=${subfolder}&type=${type}`;
  },

  // Interrupt current generation
  async interrupt(): Promise<{ status: string }> {
    return request("/interrupt", { method: "POST" });
  },

  // Clear queue
  async clear(): Promise<{ status: string }> {
    return request("/clear", { method: "POST" });
  },

  // Test ComfyUI connection
  async testComfyUI(url: string): Promise<{ success: boolean }> {
    return request("/test-comfyui", {
      method: "POST",
      body: JSON.stringify({ url }),
    });
  },

  // Get ComfyUI URL from backend
  async getComfyUIUrl(): Promise<{ url: string }> {
    return request("/comfyui-url");
  },

  // Set ComfyUI URL on backend
  async setComfyUIUrl(url: string): Promise<{ success: boolean; url: string }> {
    return request("/comfyui-url", {
      method: "POST",
      body: JSON.stringify({ url }),
    });
  },
};

export { ApiError };
