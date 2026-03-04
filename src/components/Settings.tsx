import type { ReactNode } from "react";
import { useSettings } from "../hooks/useSettings";
import { Toggle } from "./Toggle";
import type { Settings as SettingsType } from "../types";

export function Settings() {
  const { settings, loading, save, reset, testNotification } =
    useSettings();

  if (loading || !settings) {
    return (
      <div style={{ padding: 32, textAlign: "center", color: "#8888aa" }}>
        Loading...
      </div>
    );
  }

  const update = (patch: Partial<SettingsType>) => {
    save({ ...settings, ...patch });
  };

  return (
    <div style={{ padding: "24px 28px", height: "100vh", overflow: "auto" }}>
      <h1
        style={{
          fontSize: 20,
          fontWeight: 600,
          color: "#e8a849",
          marginBottom: 4,
        }}
      >
        Claude Notify
      </h1>
      <p style={{ fontSize: 13, color: "#8888aa", marginBottom: 24 }}>
        Notification companion for Claude Code
      </p>

      {/* Hook Toggles */}
      <Section title="Notification Hooks">
        <Toggle
          label="Permission Prompt"
          description="When Claude needs permission to run a tool"
          checked={settings.hooks.permission_prompt}
          onChange={(v) =>
            update({ hooks: { ...settings.hooks, permission_prompt: v } })
          }
        />
        <Toggle
          label="Idle / Waiting"
          description="When Claude is waiting for your input"
          checked={settings.hooks.idle_prompt}
          onChange={(v) =>
            update({ hooks: { ...settings.hooks, idle_prompt: v } })
          }
        />
        <Toggle
          label="Task Completed"
          description="When Claude finishes a task"
          checked={settings.hooks.stop}
          onChange={(v) =>
            update({ hooks: { ...settings.hooks, stop: v } })
          }
        />
      </Section>

      {/* Behavior */}
      <Section title="Behavior">
        <Toggle
          label="Suppress when focused"
          description="Don't notify when the terminal is in the foreground"
          checked={settings.suppress_when_focused}
          onChange={(v) => update({ suppress_when_focused: v })}
        />
        <Toggle
          label="Play sound"
          description="Play a notification sound"
          checked={settings.play_sound}
          onChange={(v) => update({ play_sound: v })}
        />
        <Toggle
          label="Start on login"
          description="Launch Claude Notify when you log in"
          checked={settings.auto_start}
          onChange={(v) => update({ auto_start: v })}
        />
      </Section>

      {/* Server */}
      <Section title="Server">
        <div
          style={{
            display: "flex",
            alignItems: "center",
            justifyContent: "space-between",
            padding: "12px 0",
          }}
        >
          <div>
            <div style={{ fontSize: 14, fontWeight: 500 }}>Port</div>
            <div style={{ fontSize: 12, color: "#8888aa", marginTop: 2 }}>
              HTTP server port (requires restart)
            </div>
          </div>
          <input
            type="number"
            value={settings.server_port}
            onChange={(e) =>
              update({ server_port: parseInt(e.target.value) || 31311 })
            }
            style={{
              width: 80,
              padding: "6px 10px",
              borderRadius: 8,
              border: "1px solid #3a3a5c",
              background: "#1a1a2e",
              color: "#d0d0e0",
              fontSize: 14,
              textAlign: "center",
            }}
          />
        </div>
      </Section>

      {/* Actions */}
      <div
        style={{
          display: "flex",
          gap: 10,
          marginTop: 24,
          paddingBottom: 24,
        }}
      >
        <Button onClick={testNotification} primary>
          Test Notification
        </Button>
        <Button onClick={reset}>Reset to Defaults</Button>
      </div>

    </div>
  );
}

function Section({
  title,
  children,
}: {
  title: string;
  children: ReactNode;
}) {
  return (
    <div style={{ marginBottom: 20 }}>
      <h2
        style={{
          fontSize: 12,
          fontWeight: 600,
          color: "#6a6a8a",
          textTransform: "uppercase",
          letterSpacing: 1,
          marginBottom: 8,
        }}
      >
        {title}
      </h2>
      <div
        style={{
          background: "#1a1a2e",
          borderRadius: 12,
          padding: "4px 16px",
          border: "1px solid #2a2a45",
        }}
      >
        {children}
      </div>
    </div>
  );
}

function Button({
  onClick,
  primary,
  children,
}: {
  onClick: () => void;
  primary?: boolean;
  children: ReactNode;
}) {
  return (
    <button
      onClick={onClick}
      style={{
        padding: "10px 20px",
        borderRadius: 10,
        border: primary ? "none" : "1px solid #3a3a5c",
        background: primary ? "#e8a849" : "transparent",
        color: primary ? "#0f0f1a" : "#d0d0e0",
        fontSize: 14,
        fontWeight: 600,
        cursor: "pointer",
        transition: "opacity 0.2s",
      }}
      onMouseEnter={(e) => (e.currentTarget.style.opacity = "0.85")}
      onMouseLeave={(e) => (e.currentTarget.style.opacity = "1")}
    >
      {children}
    </button>
  );
}
