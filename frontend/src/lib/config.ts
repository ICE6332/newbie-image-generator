const trimTrailingSlash = (value: string) => value.replace(/\/+$/, "");

export const resolveApiBase = () => {
  const apiBase = import.meta.env.VITE_API_BASE_URL?.trim();
  if (apiBase) {
    return trimTrailingSlash(apiBase);
  }

  const backendBase = import.meta.env.VITE_BACKEND_URL?.trim();
  if (backendBase) {
    const trimmed = trimTrailingSlash(backendBase);
    return trimmed.endsWith("/api") ? trimmed : `${trimmed}/api`;
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
