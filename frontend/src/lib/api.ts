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

  const contentType = response.headers.get("content-type") || "";
  const isJson = contentType.includes("application/json");

  if (!response.ok) {
    if (isJson) {
      const error = await response
        .json()
        .catch(() => ({ error: "Unknown error" }));
      throw new ApiError(response.status, error.error || "Request failed");
    }

    const text = await response.text().catch(() => "");
    const message = text
      ? `Request failed: ${text.slice(0, 200)}`
      : "Request failed";
    throw new ApiError(response.status, message);
  }

  if (!isJson) {
    const text = await response.text().catch(() => "");
    const hint = "Expected JSON but got non-JSON response.";
    const details = text ? ` Response starts with: ${text.slice(0, 120)}` : "";
    throw new ApiError(response.status, `${hint}${details}`);
  }

  return response.json().catch((err) => {
    throw new ApiError(
      response.status,
      `Failed to parse JSON response: ${err instanceof Error ? err.message : "Unknown error"}`,
    );
  });
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

  // Get available LoRAs
  async getLoras(): Promise<string[]> {
    return request("/loras");
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
    const encodedFilename = encodeURIComponent(filename);
    const encodedSubfolder = encodeURIComponent(subfolder);
    const encodedType = encodeURIComponent(type);
    return `${API_BASE}/images/${encodedFilename}?subfolder=${encodedSubfolder}&type=${encodedType}`;
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
