# claude-notify

Cross-platform desktop notification companion for [Claude Code](https://docs.anthropic.com/en/docs/claude-code).

Get notified when Claude Code needs your attention — permission prompts, idle waits, and task completions — without keeping your terminal in focus.

## Features

- **System tray app** — runs quietly in the background with a tray icon
- **Smart notifications** — only shows when your terminal is *not* focused
- **Click to focus** — click the notification to switch back to your terminal
- **Settings UI** — configure notifications, port, duration, and auto-start
- **Per-hook toggles** — enable/disable notifications per event type (permission, idle, stop)
- **HTTP API** — local server for Claude Code hook integration (no shell scripts needed)
- **Auto-start** — optional launch on login

## Install

### From GitHub Releases

Go to the [Releases](https://github.com/mgblackwater/claude-notify/releases) page and download the installer for your platform:

| Platform | File | Notes |
|----------|------|-------|
| **Windows** | `Claude Notify_x.x.x_x64-setup.exe` | NSIS installer |
| **macOS** | `Claude Notify_x.x.x_aarch64.dmg` | Drag to Applications |
| **Linux** | `.deb` or `.AppImage` | Debian package or portable |

#### Windows installer

1. Run the `.exe` — an install wizard will open
2. Choose install location (defaults to `AppData\Local\Claude Notify`)
3. Select whether to create a desktop shortcut
4. Click **Install** and wait for it to complete
5. Search "Claude Notify" in the Start Menu (`Win+S`) to launch

To update, download and run the latest installer — it replaces the previous version.

To uninstall, go to **Settings → Apps → Claude Notify → Uninstall**.

#### macOS installer

1. Open the `.dmg` file
2. Drag **Claude Notify** into the Applications folder
3. Launch from Applications or Spotlight (`Cmd+Space`)

#### Linux installer

Debian/Ubuntu:
```bash
sudo dpkg -i claude-notify_x.x.x_amd64.deb
```

Or use the `.AppImage` directly:
```bash
chmod +x claude-notify_x.x.x_amd64.AppImage
./claude-notify_x.x.x_amd64.AppImage
```

### From GitHub Actions (pre-release builds)

Every push to `master` produces installers for all platforms:

1. Go to the [Actions tab](https://github.com/mgblackwater/claude-notify/actions)
2. Click the latest successful **Build & Release** workflow run
3. Download the artifact for your platform: **windows-installer**, **macos-installer**, or **linux-installer**

### Build from Source

Requires [Node.js](https://nodejs.org/) and [Rust](https://rustup.rs/).

```bash
npm install
npm run tauri build
```

Installers will be in `src-tauri/target/release/bundle/`.

## Setup

### 1. Start Claude Notify

Launch from the Start Menu or run the installed executable. It starts minimized to the system tray.

### 2. Configure Claude Code hooks

Add to `~/.claude/settings.json`:

```json
{
  "hooks": {
    "Notification": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "curl -s -X POST http://127.0.0.1:31311/notify -H \"Content-Type: application/json\" -d \"$CLAUDE_HOOK_EVENT_JSON\""
          }
        ]
      }
    ],
    "Stop": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "curl -s -X POST http://127.0.0.1:31311/notify -H \"Content-Type: application/json\" -d \"{\\\"hook_event_name\\\":\\\"Stop\\\",\\\"cwd\\\":\\\"$(pwd)\\\"}\""
          }
        ]
      }
    ]
  }
}
```

### 3. Restart Claude Code

Restart your Claude Code session for the hooks to take effect.

## How It Works

1. Claude Code fires hook events (permission prompt, idle, stop)
2. Hook sends a POST request to `http://127.0.0.1:31311/notify`
3. Claude Notify checks if your terminal window is focused
4. If **not focused** → shows a dark-themed toast popup in the bottom-right corner
5. If **focused** → notification is suppressed (you're already looking at it)
6. Clicking the popup brings your terminal window to the foreground

## HTTP API

### `GET /health`
Returns `ok` — use to check if the app is running.

### `POST /notify`
Send a notification. Body (JSON):

```json
{
  "hook_event_name": "Notification",
  "notification_type": "permission_prompt",
  "cwd": "/path/to/project",
  "message": "Claude needs permission to run a command"
}
```

Fields:
- `notification_type` — `permission_prompt`, `idle_prompt`, or omit for general
- `hook_event_name` — `Stop` for task completion events
- `cwd` — project directory (used to extract project name)
- `message` — notification body text

## Settings

Right-click the tray icon → **Settings**, or left-click the tray icon.

| Setting | Default | Description |
|---------|---------|-------------|
| Permission prompt | On | Notify on permission prompts |
| Idle prompt | On | Notify when Claude is waiting for input |
| Task completed | On | Notify when a task finishes |
| Notification duration | 5s | How long the toast stays visible |
| Suppress when focused | On | Skip notifications when terminal is focused |
| Server port | 31311 | HTTP server port |
| Auto-start | Off | Launch on login |

## Tech Stack

- **Tauri v2** — Rust backend + web frontend
- **React 19** — Settings UI
- **Axum** — Local HTTP server
- **Platform-native focus detection** — Win32 (Windows), AppKit (macOS), X11 (Linux)

## License

MIT
