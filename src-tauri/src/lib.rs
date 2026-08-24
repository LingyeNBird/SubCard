mod model;
mod remote;
mod storage;
mod tray;

use model::{AppSnapshot, AppState, ParticipantCardData, RefreshResult, StoredConfig};
use std::{
    collections::HashSet,
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Manager, State};
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
    *state
        .participants
        .lock()
        .map_err(|_| lock_error("参与者"))? = participants.clone();
    tray::rebuild_menu(app)?;
    let refreshed_at_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    Ok(RefreshResult {
        participants,
        actionable_participant_ids,
        selected_participant_ids,
        refreshed_at_ms,
    })
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
    })
}

#[tauri::command]
async fn refresh_participants(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<RefreshResult, String> {
    let config = config_snapshot(&state)?;
    if config.base_url.is_empty() {
        return Err("请先配置 Sub2Pool 服务地址".to_string());
    }
    let token = storage::read_token()?.ok_or_else(|| "请先配置 Sub2Pool API Key".to_string())?;
    let (participants, actionable_participant_ids) =
        remote::fetch_card_data(&config.base_url, &token).await?;
    apply_participants(&app, &state, participants, actionable_participant_ids)
}

#[tauri::command]
async fn apply_participant_recommendation(
    app: AppHandle,
    state: State<'_, AppState>,
    participant_id: i64,
) -> Result<RefreshResult, String> {
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
fn hide_card(app: AppHandle) -> Result<(), String> {
    tray::hide_window(&app)
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
            app.manage(AppState::new(config));
            tray::setup(app).map_err(std::io::Error::other)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_app_snapshot,
            refresh_participants,
            apply_participant_recommendation,
            save_connection,
            hide_card,
            fit_window_to_content,
            save_card_width,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Subcard");
}
