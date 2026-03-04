import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Settings } from "../types";

export function useSettings() {
  const [settings, setSettings] = useState<Settings | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    invoke<Settings>("get_settings")
      .then(setSettings)
      .catch(console.error)
      .finally(() => setLoading(false));
  }, []);

  const save = useCallback((newSettings: Settings) => {
    setSettings(newSettings);
    // Fire and forget — don't await, don't block UI
    invoke("update_settings", { newSettings }).catch((e) =>
      console.error("Failed to save settings:", e)
    );
  }, []);

  const reset = useCallback(async () => {
    try {
      const defaults = await invoke<Settings>("reset_settings");
      setSettings(defaults);
    } catch (e) {
      console.error("Failed to reset settings:", e);
    }
  }, []);

  const testNotification = useCallback(() => {
    invoke("test_notification").catch(console.error);
  }, []);

  return { settings, loading, saving: false, save, reset, testNotification };
}
