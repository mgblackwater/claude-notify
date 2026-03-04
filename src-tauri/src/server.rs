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
    pub title: Option<String>,
    pub last_assistant_message: Option<String>,
    pub tool_name: Option<String>,
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
    let settings = Settings::load();

    let is_stop = payload.hook_event_name.as_deref() == Some("Stop");

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
        _ => if is_stop { settings.hooks.stop } else { true },
    };

    if !enabled {
        return StatusCode::OK;
    }

    // Check focus suppression
    if settings.suppress_when_focused && focus::is_terminal_focused() {
        return StatusCode::OK;
    }

    // Use title from Claude Code if provided, otherwise derive from hook type
    let title = payload.title.clone().unwrap_or_else(|| {
        match hook_type {
            "permission_prompt" => "Permission needed".to_string(),
            "idle_prompt" => "Waiting for your input".to_string(),
            "Stop" | _ if is_stop => "Task completed".to_string(),
            _ => "Claude Code".to_string(),
        }
    });

    let project = payload.cwd
        .as_deref()
        .and_then(|p| p.rsplit(&['/', '\\']).next())
        .unwrap_or("")
        .to_string();

    // Build message: use message field, then last_assistant_message, with tool context
    let mut message = payload.message.clone().unwrap_or_default();

    // For permission prompts, include which tool needs permission
    if hook_type == "permission_prompt" {
        if let Some(tool) = &payload.tool_name {
            if message.is_empty() {
                message = format!("Allow {}?", tool);
            }
        }
    }

    // For stop events, show a summary of what Claude said
    if message.is_empty() {
        if let Some(last_msg) = &payload.last_assistant_message {
            // Take first line or first 200 chars
            let summary = last_msg.lines().next().unwrap_or(last_msg);
            message = if summary.len() > 200 {
                format!("{}...", &summary[..200])
            } else {
                summary.to_string()
            };
        }
    }

    crate::show_toast_window(
        &state.app_handle,
        &title,
        &project,
        &message,
        settings.notification_duration,
    );

    StatusCode::OK
}
