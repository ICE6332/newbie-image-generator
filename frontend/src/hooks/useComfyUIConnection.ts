import { useCallback, useEffect, useState } from "react";
import { api } from "@/lib/api";
const DEFAULT_COMFYUI_URL = "http://127.0.0.1:8188";

export type ConnectionTestResult = "success" | "error" | null;

export function useComfyUIConnection() {
  const [comfyuiUrl, setComfyuiUrlState] = useState(DEFAULT_COMFYUI_URL);
  const [comfyuiConnected, setComfyuiConnected] = useState<boolean | null>(
    null,
  );
  const [testingConnection, setTestingConnection] = useState(false);
  const [testResult, setTestResult] = useState<ConnectionTestResult>(null);

  const checkComfyUIHealth = useCallback(async () => {
    try {
      const health = await api.health();
      setComfyuiConnected(health.comfyui);
    } catch {
      setComfyuiConnected(false);
    }
  }, []);

  const updateComfyUIUrl = useCallback(
    async (url: string) => {
      setComfyuiUrlState(url);
      try {
        const response = await api.setComfyUIUrl(url);
        if (response.url) {
          setComfyuiUrlState(response.url);
        }
      } catch {
        // Backend might not be available yet
      }
      await checkComfyUIHealth();
    },
    [checkComfyUIHealth],
  );

  const testConnection = useCallback(async () => {
    setTestingConnection(true);
    setTestResult(null);
    await updateComfyUIUrl(comfyuiUrl);
    try {
      let url = comfyuiUrl.trim();
      if (url && !url.startsWith("http://") && !url.startsWith("https://")) {
        url = `http://${url}`;
      }
      const result = await api.testComfyUI(url);
      setTestResult(result.success ? "success" : "error");
      setComfyuiConnected(result.success);
    } catch {
      setTestResult("error");
      setComfyuiConnected(false);
    } finally {
      setTestingConnection(false);
      setTimeout(() => setTestResult(null), 3000);
    }
  }, [comfyuiUrl, updateComfyUIUrl]);

  useEffect(() => {
    const fetchUrl = async () => {
      try {
        const response = await api.getComfyUIUrl();
        if (response.url) {
          setComfyuiUrlState(response.url);
        }
      } catch {
        // Backend might not be available yet
      }
    };
    fetchUrl();
  }, []);

  useEffect(() => {
    checkComfyUIHealth();
    const interval = window.setInterval(() => {
      checkComfyUIHealth();
    }, 5000);
    return () => window.clearInterval(interval);
  }, [checkComfyUIHealth]);

  return {
    comfyuiUrl,
    comfyuiConnected,
    testingConnection,
    testResult,
    updateComfyUIUrl,
    testConnection,
  };
}
