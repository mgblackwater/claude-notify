use axum::{extract::State as AxumState, http::StatusCode, routing::{get, post}, Json, Router};
use serde::Deserialize;
use tauri::AppHandle;

use crate::focus;
use crate::settings::Settings;

#[derive(Debug, Deserialize)]
pub struct HookPayload {
    pub hook_event_name: Option<String>,
    pub notification_type: Option<String>,
    pub cwd: Option<String>,
    pub message: Option<String>,
    pub last_assistant_message: Option<String>,
}

#[derive(Clone)]
pub struct ServerState {
    pub app_handle: AppHandle,
}

pub async fn start_server(app_handle: AppHandle, port: u16) {
    let state = ServerState { app_handle };

    let app = Router::new()
        .route("/health", get(health))
        .route("/notify", post(notify))
        .with_state(state);

    let addr = format!("127.0.0.1:{}", port);
    log::info!("Starting notification server on {}", addr);

    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            log::error!("Failed to bind to {}: {}", addr, e);
            return;
        }
    };

    if let Err(e) = axum::serve(listener, app).await {
        log::error!("Server error: {}", e);
    }
}

async fn health() -> &'static str {
    "ok"
}

async fn notify(
    AxumState(state): AxumState<ServerState>,
    Json(payload): Json<HookPayload>,
) -> StatusCode {
    // Read settings from disk — no locks, no shared state
    let settings = Settings::load();

    // Determine hook type
    let hook_type = payload.notification_type
        .as_deref()
        .or(payload.hook_event_name.as_deref())
        .unwrap_or("unknown");

    // Check if this hook type is enabled
    let enabled = match hook_type {
        "permission_prompt" => settings.hooks.permission_prompt,
        "idle_prompt" => settings.hooks.idle_prompt,
        "Stop" => settings.hooks.stop,
        _ => {
            if payload.hook_event_name.as_deref() == Some("Stop") {
                settings.hooks.stop
            } else {
                true
            }
        }
    };

    if !enabled {
        return StatusCode::OK;
    }

    // Check focus suppression
    if settings.suppress_when_focused && focus::is_terminal_focused() {
        return StatusCode::OK;
    }

    // Build notification
    let title = match hook_type {
        "permission_prompt" => "Permission needed".to_string(),
        "idle_prompt" => "Waiting for your input".to_string(),
        "Stop" => "Task completed".to_string(),
        _ => {
            if payload.hook_event_name.as_deref() == Some("Stop") {
                "Task completed".to_string()
            } else {
                "Claude Code".to_string()
            }
        }
    };

    let project = payload.cwd
        .as_deref()
        .and_then(|p| p.rsplit(&['/', '\\']).next())
        .unwrap_or("")
        .to_string();

    let message = payload.message
        .clone()
        .or_else(|| {
            payload.last_assistant_message.as_ref().map(|m| {
                if m.len() > 200 {
                    format!("{}...", &m[..200])
                } else {
                    m.clone()
                }
            })
        })
        .unwrap_or_default();

    // Show toast popup window
    crate::show_toast_window(&state.app_handle, &title, &project, &message);

    StatusCode::OK
}
