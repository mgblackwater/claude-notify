interface ToggleProps {
  label: string;
  description?: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
}

export function Toggle({ label, description, checked, onChange }: ToggleProps) {
  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
        padding: "12px 0",
        borderBottom: "1px solid #1e1e35",
      }}
    >
      <div>
        <div style={{ fontSize: 14, fontWeight: 500 }}>{label}</div>
        {description && (
          <div style={{ fontSize: 12, color: "#8888aa", marginTop: 2 }}>
            {description}
          </div>
        )}
      </div>
      <button
        onClick={() => onChange(!checked)}
        style={{
          width: 44,
          height: 24,
          borderRadius: 12,
          border: "none",
          cursor: "pointer",
          position: "relative",
          background: checked ? "#e8a849" : "#2a2a45",
          transition: "background 0.2s",
          flexShrink: 0,
        }}
      >
        <div
          style={{
            width: 18,
            height: 18,
            borderRadius: 9,
            background: "#fff",
            position: "absolute",
            top: 3,
            left: checked ? 23 : 3,
            transition: "left 0.2s",
          }}
        />
      </button>
    </div>
  );
}
