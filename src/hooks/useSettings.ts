import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Settings } from "../types";

export function useSettings() {
  const [settings, setSettings] = useState<Settings | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    invoke<Settings>("get_settings")
      .then(setSettings)
      .catch(console.error)
      .finally(() => setLoading(false));
  }, []);

  const save = useCallback(async (newSettings: Settings) => {
    setSaving(true);
    try {
      await invoke("update_settings", { newSettings });
      setSettings(newSettings);
    } catch (e) {
      console.error("Failed to save settings:", e);
    } finally {
      setSaving(false);
    }
  }, []);

  const reset = useCallback(async () => {
    setSaving(true);
    try {
      const defaults = await invoke<Settings>("reset_settings");
      setSettings(defaults);
    } catch (e) {
      console.error("Failed to reset settings:", e);
    } finally {
      setSaving(false);
    }
  }, []);

  const testNotification = useCallback(async () => {
    await invoke("test_notification");
  }, []);

  return { settings, loading, saving, save, reset, testNotification };
}
