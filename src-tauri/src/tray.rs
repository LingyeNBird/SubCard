use crate::{
    model::{AppState, DisplayMode, ParticipantCardData},
    storage,
};
use tauri::{
    menu::{CheckMenuItemBuilder, Menu, MenuItemBuilder, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_positioner::{Position, WindowExt};

const TRAY_ID: &str = "subcard-tray";
const SHOW_ID: &str = "show-card";
const REFRESH_ID: &str = "refresh";
const SETTINGS_ID: &str = "settings";
const AUTOSTART_ID: &str = "autostart";
const MINIMAL_MODE_ID: &str = "minimal-mode";
const HIDE_ID: &str = "hide-card";
const QUIT_ID: &str = "quit";
const PARTICIPANT_PREFIX: &str = "participant:";

fn lock_error(name: &str) -> String {
    format!("{name}状态暂时不可用")
}

fn clamp_axis_to_work_area(
    position: i32,
    window_size: u32,
    work_area_position: i32,
    work_area_size: u32,
) -> i32 {
    let minimum = i64::from(work_area_position);
    let maximum = (minimum + i64::from(work_area_size) - i64::from(window_size)).max(minimum);
    i64::from(position).clamp(minimum, maximum) as i32
}

fn far_edge_axis_position(window_size: u32, work_area_position: i32, work_area_size: u32) -> i32 {
    let minimum = i64::from(work_area_position);
    (minimum + i64::from(work_area_size) - i64::from(window_size))
        .max(minimum)
        .clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}

fn anchored_axis_position(position: i32, old_size: u32, new_size: u32) -> i32 {
    (i64::from(position) + i64::from(old_size) - i64::from(new_size))
        .clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}

fn clamp_window_to_work_area(window: &tauri::WebviewWindow) -> Result<(), String> {
    let Some(monitor) = window
        .current_monitor()
        .map_err(|error| format!("读取卡片所在屏幕失败：{error}"))?
    else {
        return Ok(());
    };
    let position = window
        .outer_position()
        .map_err(|error| format!("读取卡片位置失败：{error}"))?;
    let size = window
        .outer_size()
        .map_err(|error| format!("读取卡片尺寸失败：{error}"))?;
    let work_area = monitor.work_area();
    let x = clamp_axis_to_work_area(
        position.x,
        size.width,
        work_area.position.x,
        work_area.size.width,
    );
    let y = clamp_axis_to_work_area(
        position.y,
        size.height,
        work_area.position.y,
        work_area.size.height,
    );
    if x != position.x || y != position.y {
        window
            .set_position(tauri::PhysicalPosition::new(x, y))
            .map_err(|error| format!("修正卡片窗口位置失败：{error}"))?;
    }
    Ok(())
}

fn align_window_to_work_area_right_edge(window: &tauri::WebviewWindow) -> Result<(), String> {
    let Some(monitor) = window
        .current_monitor()
        .map_err(|error| format!("读取卡片所在屏幕失败：{error}"))?
    else {
        return Ok(());
    };
    let position = window
        .outer_position()
        .map_err(|error| format!("读取卡片位置失败：{error}"))?;
    let size = window
        .outer_size()
        .map_err(|error| format!("读取卡片尺寸失败：{error}"))?;
    let work_area = monitor.work_area();
    let aligned_position = tauri::PhysicalPosition::new(
        far_edge_axis_position(size.width, work_area.position.x, work_area.size.width),
        clamp_axis_to_work_area(
            position.y,
            size.height,
            work_area.position.y,
            work_area.size.height,
        ),
    );
    if aligned_position != position {
        window
            .set_position(aligned_position)
            .map_err(|error| format!("贴靠卡片窗口右边缘失败：{error}"))?;
    }
    Ok(())
}

pub fn fit_window_to_content(app: &AppHandle, width: f64, height: f64) -> Result<(), String> {
    if !width.is_finite() || !height.is_finite() || width <= 0.0 || height <= 0.0 {
        return Err("卡片内容尺寸无效".to_string());
    }
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "卡片窗口尚未就绪".to_string())?;
    let (maximum_width, maximum_height) = window
        .current_monitor()
        .map_err(|error| format!("读取卡片所在屏幕失败：{error}"))?
        .map(|monitor| {
            let scale_factor = monitor.scale_factor();
            let work_area = monitor.work_area();
            (
                f64::from(work_area.size.width) / scale_factor,
                f64::from(work_area.size.height) / scale_factor,
            )
        })
        .unwrap_or((width, height));
    let visible = window
        .is_visible()
        .map_err(|error| format!("读取卡片窗口状态失败：{error}"))?;
    let anchor = if visible {
        Some((
            window
                .outer_position()
                .map_err(|error| format!("读取卡片位置失败：{error}"))?,
            window
                .outer_size()
                .map_err(|error| format!("读取卡片尺寸失败：{error}"))?,
        ))
    } else {
        None
    };
    window
        .set_size(tauri::LogicalSize::new(
            width.min(maximum_width),
            height.min(maximum_height),
        ))
        .map_err(|error| format!("调整卡片窗口尺寸失败：{error}"))?;
    if let Some((position, old_size)) = anchor {
        let new_size = window
            .outer_size()
            .map_err(|error| format!("读取调整后的卡片尺寸失败：{error}"))?;
        let anchored_position = tauri::PhysicalPosition::new(
            anchored_axis_position(position.x, old_size.width, new_size.width),
            anchored_axis_position(position.y, old_size.height, new_size.height),
        );
        if anchored_position != position {
            window
                .set_position(anchored_position)
                .map_err(|error| format!("保持卡片边缘位置失败：{error}"))?;
        }
        clamp_window_to_work_area(&window)?;
    }
    Ok(())
}

pub fn rebuild_menu(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<AppState>();
    let participants = state
        .participants
        .lock()
        .map_err(|_| lock_error("参与者"))?
        .clone();
    let (selected, minimal_mode) = {
        let config = state.config.lock().map_err(|_| lock_error("配置"))?;
        (config.selected_participant_ids.clone(), config.minimal_mode)
    };
    let autostart_enabled = app.autolaunch().is_enabled().unwrap_or(false);
    let menu = build_menu(
        app,
        &participants,
        &selected,
        minimal_mode,
        autostart_enabled,
    )
    .map_err(|error| format!("创建托盘菜单失败：{error}"))?;
    let tray = app
        .tray_by_id(TRAY_ID)
        .ok_or_else(|| "托盘图标尚未就绪".to_string())?;
    tray.set_menu(Some(menu))
        .map_err(|error| format!("更新托盘菜单失败：{error}"))
}

fn build_menu(
    app: &AppHandle,
    participants: &[ParticipantCardData],
    selected: &[i64],
    minimal_mode: bool,
    autostart_enabled: bool,
) -> tauri::Result<Menu<tauri::Wry>> {
    let title = MenuItemBuilder::with_id("title", "Subcard")
        .enabled(false)
        .build(app)?;
    let show = MenuItemBuilder::with_id(SHOW_ID, "显示卡片").build(app)?;
    let participant_title = MenuItemBuilder::with_id("participant-title", "显示参与者")
        .enabled(false)
        .build(app)?;
    let refresh = MenuItemBuilder::with_id(REFRESH_ID, "立即刷新").build(app)?;
    let settings = MenuItemBuilder::with_id(SETTINGS_ID, "连接设置…").build(app)?;
    let minimal_mode_item = CheckMenuItemBuilder::with_id(MINIMAL_MODE_ID, "极简模式")
        .checked(minimal_mode)
        .build(app)?;
    let autostart = CheckMenuItemBuilder::with_id(AUTOSTART_ID, "开机启动")
        .checked(autostart_enabled)
        .build(app)?;
    let hide = MenuItemBuilder::with_id(HIDE_ID, "关闭卡片").build(app)?;
    let quit = MenuItemBuilder::with_id(QUIT_ID, "退出 Subcard").build(app)?;
    let separator = || PredefinedMenuItem::separator(app);

    let menu = Menu::new(app)?;
    menu.append(&title)?;
    menu.append(&show)?;
    menu.append(&separator()?)?;
    menu.append(&participant_title)?;
    if participants.is_empty() {
        let empty = MenuItemBuilder::with_id("participant-empty", "尚未加载参与者")
            .enabled(false)
            .build(app)?;
        menu.append(&empty)?;
    } else {
        for participant in participants {
            let label = if participant.enabled {
                participant.name.clone()
            } else {
                format!("{}（已停用）", participant.name)
            };
            let item = CheckMenuItemBuilder::with_id(
                format!("{PARTICIPANT_PREFIX}{}", participant.id),
                label,
            )
            .checked(selected.contains(&participant.id))
            .build(app)?;
            menu.append(&item)?;
        }
    }
    menu.append(&separator()?)?;
    menu.append(&refresh)?;
    menu.append(&settings)?;
    menu.append(&minimal_mode_item)?;
    menu.append(&autostart)?;
    menu.append(&separator()?)?;
    menu.append(&hide)?;
    menu.append(&quit)?;
    Ok(menu)
}

fn set_display_mode(app: &AppHandle, mode: DisplayMode) -> Result<(), String> {
    let state = app.state::<AppState>();
    *state.display_mode.lock().map_err(|_| lock_error("窗口"))? = mode.clone();
    app.emit("display-mode", mode)
        .map_err(|error| format!("更新窗口模式失败：{error}"))
}

pub fn show_window(app: &AppHandle, mode: DisplayMode) -> Result<(), String> {
    set_display_mode(app, mode)?;
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "卡片窗口尚未就绪".to_string())?;
    window
        .move_window(Position::TrayCenter)
        .map_err(|error| format!("定位卡片窗口失败：{error}"))?;
    align_window_to_work_area_right_edge(&window)?;
    window
        .show()
        .map_err(|error| format!("显示卡片窗口失败：{error}"))?;
    window
        .set_focus()
        .map_err(|error| format!("聚焦卡片窗口失败：{error}"))?;
    app.emit("card-visibility", true)
        .map_err(|error| format!("同步卡片状态失败：{error}"))
}

pub fn hide_window(app: &AppHandle) -> Result<(), String> {
    app.get_webview_window("main")
        .ok_or_else(|| "卡片窗口尚未就绪".to_string())?
        .hide()
        .map_err(|error| format!("关闭卡片失败：{error}"))?;
    app.emit("card-visibility", false)
        .map_err(|error| format!("同步卡片状态失败：{error}"))
}

fn toggle_window(app: &AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "卡片窗口尚未就绪".to_string())?;
    if window
        .is_visible()
        .map_err(|error| format!("读取卡片窗口状态失败：{error}"))?
    {
        hide_window(app)
    } else {
        show_window(app, DisplayMode::Card)
    }
}

fn toggle_participant(app: &AppHandle, participant_id: i64) -> Result<(), String> {
    let state = app.state::<AppState>();
    let selected = {
        let mut config = state.config.lock().map_err(|_| lock_error("配置"))?;
        config.selection_initialized = true;
        if let Some(index) = config
            .selected_participant_ids
            .iter()
            .position(|id| *id == participant_id)
        {
            config.selected_participant_ids.remove(index);
        } else {
            config.selected_participant_ids.push(participant_id);
        }
        storage::save_config(app, &config)?;
        config.selected_participant_ids.clone()
    };
    rebuild_menu(app)?;
    app.emit("selection-changed", selected)
        .map_err(|error| format!("同步参与者选择失败：{error}"))
}

fn toggle_minimal_mode(app: &AppHandle) -> Result<(), String> {
    let minimal_mode = {
        let state = app.state::<AppState>();
        let mut config = state.config.lock().map_err(|_| lock_error("配置"))?;
        config.minimal_mode = !config.minimal_mode;
        storage::save_config(app, &config)?;
        config.minimal_mode
    };
    rebuild_menu(app)?;
    app.emit("minimal-mode-changed", minimal_mode)
        .map_err(|error| format!("同步极简模式失败：{error}"))
}

fn toggle_autostart(app: &AppHandle) -> Result<(), String> {
    let manager = app.autolaunch();
    if manager
        .is_enabled()
        .map_err(|error| format!("读取开机启动状态失败：{error}"))?
    {
        manager
            .disable()
            .map_err(|error| format!("关闭开机启动失败：{error}"))?;
    } else {
        manager
            .enable()
            .map_err(|error| format!("开启开机启动失败：{error}"))?;
    }
    rebuild_menu(app)
}

fn handle_menu_event(app: &AppHandle, id: &str) -> Result<(), String> {
    if let Some(raw_id) = id.strip_prefix(PARTICIPANT_PREFIX) {
        let participant_id = raw_id
            .parse::<i64>()
            .map_err(|_| "参与者菜单项无效".to_string())?;
        return toggle_participant(app, participant_id);
    }
    match id {
        SHOW_ID => show_window(app, DisplayMode::Card),
        REFRESH_ID => app
            .emit("refresh-requested", ())
            .map_err(|error| format!("请求刷新失败：{error}")),
        SETTINGS_ID => show_window(app, DisplayMode::Settings),
        MINIMAL_MODE_ID => toggle_minimal_mode(app),
        AUTOSTART_ID => toggle_autostart(app),
        HIDE_ID => hide_window(app),
        QUIT_ID => {
            app.exit(0);
            Ok(())
        }
        _ => Ok(()),
    }
}

pub fn setup(app: &tauri::App) -> Result<(), String> {
    let state = app.state::<AppState>();
    let (selected, minimal_mode) = {
        let config = state.config.lock().map_err(|_| lock_error("配置"))?;
        (config.selected_participant_ids.clone(), config.minimal_mode)
    };
    let autostart_enabled = app.autolaunch().is_enabled().unwrap_or(false);
    let menu = build_menu(
        app.handle(),
        &[],
        &selected,
        minimal_mode,
        autostart_enabled,
    )
    .map_err(|error| format!("创建托盘菜单失败：{error}"))?;
    let icon = app
        .default_window_icon()
        .cloned()
        .ok_or_else(|| "应用图标不可用".to_string())?;
    TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| {
            if let Err(error) = handle_menu_event(app, event.id().as_ref()) {
                let _ = app.emit("desktop-error", error);
            }
        })
        .on_tray_icon_event(|tray, event| {
            tauri_plugin_positioner::on_tray_event(tray.app_handle(), &event);
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                if let Err(error) = toggle_window(tray.app_handle()) {
                    let _ = tray.app_handle().emit("desktop-error", error);
                }
            }
        })
        .build(app)
        .map_err(|error| format!("创建托盘图标失败：{error}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{anchored_axis_position, clamp_axis_to_work_area, far_edge_axis_position};

    #[test]
    fn clamps_window_inside_work_area_edges() {
        assert_eq!(clamp_axis_to_work_area(-40, 752, 0, 1920), 0);
        assert_eq!(clamp_axis_to_work_area(1500, 752, 0, 1920), 1168);
        assert_eq!(clamp_axis_to_work_area(400, 752, 0, 1920), 400);
        assert_eq!(clamp_axis_to_work_area(-900, 752, -1280, 1280), -900);
    }

    #[test]
    fn anchors_oversized_window_at_work_area_start() {
        assert_eq!(clamp_axis_to_work_area(200, 900, 100, 800), 100);
    }

    #[test]
    fn preserves_far_edge_when_window_size_changes() {
        assert_eq!(anchored_axis_position(1168, 752, 900), 1020);
        assert_eq!(anchored_axis_position(1020, 900, 640), 1280);
        assert_eq!(anchored_axis_position(700, 300, 240), 760);
    }

    #[test]
    fn aligns_reopened_window_to_work_area_far_edge() {
        assert_eq!(far_edge_axis_position(640, 0, 1920), 1280);
        assert_eq!(far_edge_axis_position(640, -1280, 1280), -640);
        assert_eq!(far_edge_axis_position(900, 100, 800), 100);
    }
}
