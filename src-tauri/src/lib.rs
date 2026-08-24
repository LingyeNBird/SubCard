mod model;
mod remote;
mod storage;
mod tray;

use model::{AppSnapshot, AppState, ParticipantCardData, RefreshResult, StoredConfig};
use std::{
    collections::HashSet,
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};

const MIN_CARD_WIDTH: f64 = 480.0;

fn lock_error(name: &str) -> String {
    format!("{name}状态暂时不可用")
}

fn config_snapshot(state: &AppState) -> Result<StoredConfig, String> {
    state
        .config
        .lock()
        .map_err(|_| lock_error("配置"))
        .map(|config| config.clone())
}

fn cached_refresh_snapshot(state: &AppState) -> Result<Option<RefreshResult>, String> {
    state
        .cached_refresh
        .lock()
        .map_err(|_| lock_error("卡片缓存"))
        .map(|cached| cached.clone())
}

fn apply_participants(
    app: &AppHandle,
    state: &AppState,
    participants: Vec<ParticipantCardData>,
    actionable_participant_ids: Vec<i64>,
) -> Result<RefreshResult, String> {
    let selected_participant_ids = {
        let valid_ids = participants
            .iter()
            .map(|participant| participant.id)
            .collect::<HashSet<_>>();
        let mut config = state.config.lock().map_err(|_| lock_error("配置"))?;
        if config.selection_initialized {
            config
                .selected_participant_ids
                .retain(|id| valid_ids.contains(id));
        } else {
            config.selected_participant_ids = participants
                .iter()
                .filter(|participant| participant.enabled)
                .map(|participant| participant.id)
                .collect();
            if config.selected_participant_ids.is_empty() {
                config.selected_participant_ids = participants
                    .iter()
                    .map(|participant| participant.id)
                    .collect();
            }
            config.selection_initialized = true;
        }
        storage::save_config(app, &config)?;
        config.selected_participant_ids.clone()
    };
    let refreshed_at_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    let result = RefreshResult {
        participants,
        actionable_participant_ids,
        selected_participant_ids,
        refreshed_at_ms,
    };
    *state
        .cached_refresh
        .lock()
        .map_err(|_| lock_error("卡片缓存"))? = Some(result.clone());
    tray::rebuild_menu(app)?;
    Ok(result)
}

#[tauri::command]
fn get_app_snapshot(app: AppHandle, state: State<'_, AppState>) -> Result<AppSnapshot, String> {
    let config = config_snapshot(&state)?;
    let configured = !config.base_url.is_empty() && storage::read_token()?.is_some();
    let display_mode = state
        .display_mode
        .lock()
        .map_err(|_| lock_error("窗口"))?
        .clone();
    let cached_refresh = cached_refresh_snapshot(&state)?;
    Ok(AppSnapshot {
        configured,
        base_url: config.base_url,
        selected_participant_ids: config.selected_participant_ids,
        autostart_enabled: app.autolaunch().is_enabled().unwrap_or(false),
        minimal_mode: config.minimal_mode,
        card_width: config.card_width,
        display_mode,
        visible: app
            .get_webview_window("main")
            .and_then(|window| window.is_visible().ok())
            .unwrap_or(false),
        cached_refresh,
    })
}

async fn refresh_participants_inner(
    app: &AppHandle,
    state: &AppState,
) -> Result<RefreshResult, String> {
    let observed_refresh_at = cached_refresh_snapshot(state)?
        .as_ref()
        .map(|cached| cached.refreshed_at_ms);
    let _refresh = state.refresh_lifecycle.lock().await;
    let current_cache = cached_refresh_snapshot(state)?;
    if current_cache.as_ref().map(|cached| cached.refreshed_at_ms) != observed_refresh_at {
        if let Some(cached) = current_cache {
            return Ok(cached);
        }
    }
    let config = config_snapshot(state)?;
    if config.base_url.is_empty() {
        return Err("请先配置 Sub2Pool 服务地址".to_string());
    }
    let token = storage::read_token()?.ok_or_else(|| "请先配置 Sub2Pool API Key".to_string())?;
    let (participants, actionable_participant_ids) =
        remote::fetch_card_data(&config.base_url, &token).await?;
    apply_participants(app, state, participants, actionable_participant_ids)
}

pub(crate) fn refresh_participants_in_background(app: &AppHandle) {
    let app = app.clone();
    drop(tauri::async_runtime::spawn(async move {
        let state = app.state::<AppState>();
        if let Err(error) = refresh_participants_inner(&app, &state).await {
            if app.get_webview_window("main").is_some() {
                let _ = app.emit("desktop-error", error);
            }
        }
    }));
}

#[tauri::command]
async fn refresh_participants(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<RefreshResult, String> {
    refresh_participants_inner(&app, &state).await
}

#[tauri::command]
async fn apply_participant_recommendation(
    app: AppHandle,
    state: State<'_, AppState>,
    participant_id: i64,
) -> Result<RefreshResult, String> {
    let _refresh = state.refresh_lifecycle.lock().await;
    let config = config_snapshot(&state)?;
    if config.base_url.is_empty() {
        return Err("请先配置 Sub2Pool 服务地址".to_string());
    }
    let token = storage::read_token()?.ok_or_else(|| "请先配置 Sub2Pool API Key".to_string())?;
    remote::apply_participant_recommendation(&config.base_url, &token, participant_id).await?;
    let (participants, actionable_participant_ids) =
        remote::fetch_card_data(&config.base_url, &token).await?;
    apply_participants(&app, &state, participants, actionable_participant_ids)
}

#[tauri::command]
async fn save_connection(
    app: AppHandle,
    state: State<'_, AppState>,
    base_url: String,
    token: String,
) -> Result<RefreshResult, String> {
    let normalized_url = remote::normalize_base_url(&base_url)?;
    let _refresh = state.refresh_lifecycle.lock().await;
    let trimmed_token = token.trim();
    let effective_token = if trimmed_token.is_empty() {
        storage::read_token()?.ok_or_else(|| "请输入 Sub2Pool API Key".to_string())?
    } else {
        trimmed_token.to_string()
    };
    let (participants, actionable_participant_ids) =
        remote::fetch_card_data(&normalized_url, &effective_token).await?;
    if !trimmed_token.is_empty() {
        storage::save_token(trimmed_token)?;
    }
    {
        let mut config = state.config.lock().map_err(|_| lock_error("配置"))?;
        if config.base_url != normalized_url {
            config.selected_participant_ids.clear();
            config.selection_initialized = false;
        }
        config.base_url = normalized_url;
    }
    apply_participants(&app, &state, participants, actionable_participant_ids)
}

#[tauri::command]
fn close_card(app: AppHandle) {
    tray::close_window(&app);
}

#[tauri::command]
fn fit_window_to_content(app: AppHandle, width: f64, height: f64) -> Result<(), String> {
    tray::fit_window_to_content(&app, width, height)
}

#[tauri::command]
fn save_card_width(app: AppHandle, state: State<'_, AppState>, width: f64) -> Result<(), String> {
    if !width.is_finite() || width < MIN_CARD_WIDTH {
        return Err(format!("卡片宽度不能小于 {MIN_CARD_WIDTH:.0}"));
    }
    let mut config = state.config.lock().map_err(|_| lock_error("配置"))?;
    config.card_width = Some(width.round());
    storage::save_config(&app, &config)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .setup(|app| {
            let config = storage::load_config(app.handle()).map_err(std::io::Error::other)?;
            let refresh_on_start = !config.base_url.is_empty();
            app.manage(AppState::new(config));
            tray::setup(app).map_err(std::io::Error::other)?;
            if refresh_on_start {
                refresh_participants_in_background(app.handle());
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                tray::close_window(window.app_handle());
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_app_snapshot,
            refresh_participants,
            apply_participant_recommendation,
            save_connection,
            close_card,
            fit_window_to_content,
            save_card_width,
        ])
        .build(tauri::generate_context!())
        .expect("error while building Subcard")
        .run(|_app, event| {
            if let tauri::RunEvent::ExitRequested { code, api, .. } = event {
                if code.is_none() {
                    api.prevent_exit();
                }
            }
        });
}
