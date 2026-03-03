# claude-notify

Cross-platform desktop notification companion for [Claude Code](https://docs.anthropic.com/en/docs/claude-code).

Get notified when Claude Code needs your attention — permission prompts, idle waits, and task completions — without keeping your terminal in focus.

## Features

- **Smart notifications** — only shows when your terminal is *not* focused
- **Click to focus** — click the notification to switch back to your terminal
- **Hook integration** — works with Claude Code's hook system (Notification, Stop events)
- **Project context** — shows project name and the actual question/message
- **Auto-dismiss** — fades out after 5 seconds

## Current Status

**Windows scripts are working** (Git Bash / Windows Terminal). A cross-platform Tauri v2 app with system tray and settings UI is planned — see [Roadmap](#roadmap).

## Quick Start (Windows)

### 1. Install scripts

```bash
# Copy scripts to ~/bin
mkdir -p ~/bin
cp scripts/windows/claude-notify.sh ~/bin/
cp scripts/windows/claude-notify.ps1 ~/bin/
chmod +x ~/bin/claude-notify.sh

# Add ~/bin to PATH (add to ~/.bashrc)
if [[ ":$PATH:" != *":$HOME/bin:"* ]]; then
  export PATH="$HOME/bin:$PATH"
fi
```

### 2. Configure Claude Code hooks

Add to `~/.claude/settings.json`:

```json
{
  "hooks": {
    "Notification": [
      {
        "matcher": "permission_prompt",
        "hooks": [
          {
            "type": "command",
            "command": "claude-notify.sh \"Permission needed\""
          }
        ]
      },
      {
        "matcher": "idle_prompt",
        "hooks": [
          {
            "type": "command",
            "command": "claude-notify.sh \"Waiting for your input\""
          }
        ]
      }
    ],
    "Stop": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "claude-notify.sh \"Task completed\""
          }
        ]
      }
    ]
  }
}
```

### 3. Restart Claude Code

Restart your Claude Code session for the hooks to take effect.

### 4. Test

Switch to another app — notifications will appear when Claude Code needs attention.

Manual test:

```bash
echo '{"cwd":"/your/project","message":"Test notification"}' | claude-notify.sh "Test"
```

## How It Works

1. Claude Code fires hook events (permission prompt, idle, stop)
2. Hook calls `claude-notify.sh` with event JSON via stdin
3. Script checks if your terminal window (Windows Terminal / mintty) is focused
4. If **not focused** → shows a dark-themed WPF popup in the bottom-right corner
5. If **focused** → notification is suppressed (you're already looking at it)
6. Clicking the popup brings your terminal window to the foreground

## Supported Terminals (Windows)

- Windows Terminal
- mintty (Git Bash standalone)

## Roadmap

### v2 — Cross-Platform Tauri App

A full desktop app built with **Tauri v2** (Rust + React) is planned:

- **System tray** — runs as a background utility with tray icon
- **Settings UI** — configure notification preferences from a settings window
- **Per-hook toggles** — enable/disable notifications per event type
- **Cross-platform** — Windows, macOS, Linux
- **Focus detection** — platform-native (Win32, AppKit, X11/Wayland)
- **Click-to-focus** — activate the correct terminal window
- **Native notifications** — OS-native toast/notification center integration
- **Auto-start** — optional launch on login
- **Sound** — optional notification sound
- **HTTP API** — local server for hook integration (no shell scripts needed)

### Architecture (Planned)

```
claude-notify/
├── src-tauri/                    # Rust backend
│   ├── src/
│   │   ├── main.rs              # Entry point
│   │   ├── lib.rs               # App setup, tray, commands
│   │   ├── server.rs            # Local HTTP server (axum)
│   │   ├── notification.rs      # Notification logic
│   │   ├── focus.rs             # Focus detection (platform-specific)
│   │   └── settings.rs          # Settings management
│   └── Cargo.toml
├── src/                          # React frontend (settings UI)
│   ├── App.tsx
│   ├── components/
│   │   ├── Settings.tsx
│   │   └── HookToggle.tsx
│   └── main.tsx
├── scripts/                      # Current shell-based implementation
│   └── windows/
└── package.json
```

## Contributing

Contributions welcome! See the roadmap above for planned features.

## License

MIT
