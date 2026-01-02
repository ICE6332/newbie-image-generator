import { useCallback, useEffect, useState } from "react";
import { api } from "@/lib/api";
import { getComfyUIUrl, setComfyUIUrl } from "@/lib/config";

export type ConnectionTestResult = "success" | "error" | null;

export function useComfyUIConnection() {
  const [comfyuiUrl, setComfyuiUrlState] = useState(getComfyUIUrl());
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

  const syncComfyUIUrl = useCallback(
    async (url: string) => {
      setComfyuiUrlState(url);
      setComfyUIUrl(url);
      try {
        await api.setComfyUIUrl(url);
      } catch {
        // Backend might not be available yet
      }
    },
    [],
  );

  const updateComfyUIUrl = useCallback(
    async (url: string) => {
      await syncComfyUIUrl(url);
      await checkComfyUIHealth();
    },
    [checkComfyUIHealth, syncComfyUIUrl],
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
    syncComfyUIUrl(getComfyUIUrl());
  }, [syncComfyUIUrl]);

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
