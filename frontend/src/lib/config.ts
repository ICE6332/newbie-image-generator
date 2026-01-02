const trimTrailingSlash = (value: string) => value.replace(/\/+$/, "");

const COMFYUI_URL_KEY = "comfyui_url";
const DEFAULT_COMFYUI_URL = "http://127.0.0.1:8188";

export const getComfyUIUrl = (): string => {
  const url = localStorage.getItem(COMFYUI_URL_KEY) || DEFAULT_COMFYUI_URL;
  // Ensure URL has protocol
  if (url && !url.startsWith("http://") && !url.startsWith("https://")) {
    return `http://${url}`;
  }
  return url;
};

export const setComfyUIUrl = (url: string): void => {
  localStorage.setItem(COMFYUI_URL_KEY, trimTrailingSlash(url));
};

export const resolveApiBase = () => {
  const apiBase = import.meta.env.VITE_API_BASE_URL?.trim();
  if (apiBase) {
    return trimTrailingSlash(apiBase);
  }

  const backendBase = import.meta.env.VITE_BACKEND_URL?.trim();
  if (backendBase) {
    return `${trimTrailingSlash(backendBase)}/api`;
  }

  return "/api";
};

export const resolveWsUrl = () => {
  const wsUrl = import.meta.env.VITE_WS_URL?.trim();
  if (wsUrl) {
    return wsUrl;
  }

  const backendBase = import.meta.env.VITE_BACKEND_URL?.trim();
  if (backendBase) {
    const url = new URL(backendBase);
    url.protocol = url.protocol === "https:" ? "wss:" : "ws:";
    return `${url.origin}/ws`;
  }

  const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
  return `${protocol}//${window.location.host}/ws`;
};
