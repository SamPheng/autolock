//! 自动锁屏应用入口
//!
//! 负责：单实例检测、定时器初始化、系统托盘、窗口生命周期管理、中文字体加载

#![windows_subsystem = "windows"]

mod app;
mod platform;
mod timer;

use app::AutolockApp;
use eframe::{NativeOptions, egui};
use platform::{
    force_exit, listen_show_window, monitor_session_events, notify_existing_instance,
    show_main_window, trigger_lock, try_single_instance,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use timer::{Timer, start_timer_thread};
use tray_icon::menu::{Menu, MenuId, MenuItem};
use tray_icon::TrayIconEvent;

/// 托盘菜单项 ID 容器，用于后台线程匹配菜单事件
pub(crate) struct TrayIds {
    pub(crate) show_id: MenuId,
    pub(crate) quit_id: MenuId,
}

/// 全局标志：从托盘恢复窗口后，跳过最小化拦截逻辑
///
/// 窗口恢复时 eframe 内部 minimized 状态可能残留一帧，导致窗口立即被重新隐藏。
/// 设置此标志后，`app.rs` 的 update 循环会优先处理"恢复窗口"指令，而非拦截最小化。
pub(crate) static SKIP_INTERCEPT: AtomicBool = AtomicBool::new(false);

/// 全局 egui::Context，供后台线程唤醒渲染循环
pub(crate) static EGUI_CTX: std::sync::OnceLock<egui::Context> = std::sync::OnceLock::new();

/// 从后台线程请求显示主窗口
///
/// 三步操作：设置跳过拦截标志 → Win32 API 恢复窗口 → 唤醒 eframe 渲染循环
fn request_show_window() {
    SKIP_INTERCEPT.store(true, Ordering::SeqCst);
    show_main_window();
    if let Some(ctx) = EGUI_CTX.get() {
        ctx.request_repaint();
    }
}

fn main() {
    // ── 单实例检测 ──────────────────────────────────────
    // 如果已有实例运行，通过命名事件通知其显示窗口，然后退出
    if try_single_instance().is_err() {
        notify_existing_instance();
        return;
    }

    // ── 图标加载 ────────────────────────────────────────
    // 同一份 ico 数据用于：eframe 窗口图标 + 系统托盘图标
    let icon_data = include_bytes!("icon.ico");
    let icon_image = image::load_from_memory(icon_data)
        .expect("Failed to load icon")
        .to_rgba8();
    let (width, height) = icon_image.dimensions();

    let icon = egui::IconData {
        rgba: icon_image.into_raw(),
        width,
        height,
    };

    // ── 定时器初始化 ────────────────────────────────────
    let timer = Timer::new(25); // 默认 25 分钟

    // 定时器到期回调：锁定工作站
    timer.lock().unwrap().set_callback(move || {
        trigger_lock();
    });

    // 后台线程每秒检查定时器是否到期
    start_timer_thread(timer.clone());

    // 会话监控：屏幕解锁后重置计时器（用户回来后重新倒计时）
    let timer_clone = timer.clone();
    monitor_session_events(move || {
        timer_clone.lock().unwrap().reset();
    });

    // 启动时自动开始倒计时
    timer.lock().unwrap().start();

    // ── eframe 窗口配置 ─────────────────────────────────
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([280.0, 220.0])
            .with_min_inner_size([260.0, 200.0])
            .with_icon(icon),
        ..Default::default()
    };

    let _ = eframe::run_native(
        "自动锁屏",
        options,
        Box::new(move |cc| {
            // 加载系统中文字体，解决中文显示方块问题
            setup_chinese_fonts(cc);

            // ── Catppuccin Mocha 深色主题 ─────────────────
            let mut style = (*cc.egui_ctx.style()).clone();
            style.spacing.item_spacing = egui::vec2(6.0, 4.0);
            style.spacing.button_padding = egui::vec2(10.0, 4.0);

            let mut visuals = egui::Visuals::dark();
            visuals.panel_fill = egui::Color32::from_rgb(30, 30, 46);       // Base
            visuals.window_fill = egui::Color32::from_rgb(30, 30, 46);
            visuals.override_text_color = Some(egui::Color32::from_rgb(205, 214, 244)); // Text
            visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(69, 71, 90)); // Surface0
            visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(49, 50, 68);     // Surface1
            visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(88, 91, 112)); // Surface2
            visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(205, 214, 244));
            visuals.widgets.active.bg_fill = egui::Color32::from_rgb(137, 180, 250);     // Blue
            visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);
            visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(116, 199, 236);    // Sapphire
            visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);
            visuals.selection.bg_fill = egui::Color32::from_rgb(137, 180, 250);           // Blue
            visuals.selection.stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);
            style.visuals = visuals;

            cc.egui_ctx.set_style(style);

            // 存储 egui::Context 到全局，供后台线程唤醒渲染循环
            EGUI_CTX.set(cc.egui_ctx.clone()).ok();

            // ── 系统托盘 ─────────────────────────────────
            let (tray, tray_ids) = create_tray_icon(icon_data);

            // 后台轮询线程：监听托盘菜单点击和图标双击事件
            let show_id = tray_ids.show_id.clone();
            let quit_id = tray_ids.quit_id.clone();
            std::thread::spawn(move || {
                loop {
                    // 菜单事件（显示窗口 / 退出）
                    if let Ok(event) = tray_icon::menu::MenuEvent::receiver().try_recv() {
                        if event.id == show_id {
                            request_show_window();
                        } else if event.id == quit_id {
                            force_exit();
                        }
                    }

                    // 托盘图标双击事件
                    if let Ok(event) = TrayIconEvent::receiver().try_recv() {
                        if let TrayIconEvent::DoubleClick { .. } = event {
                            request_show_window();
                        }
                    }

                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
            });

            // 监听来自新实例的显示窗口请求（命名事件）
            listen_show_window(move || {
                request_show_window();
            });

            Ok(Box::new(AutolockApp::new(timer, tray)))
        }),
    );
}

/// 创建系统托盘图标及右键菜单
///
/// 返回托盘实例（需由 AutolockApp 持有以维持生命周期）和菜单项 ID
fn create_tray_icon(icon_data: &[u8]) -> (tray_icon::TrayIcon, TrayIds) {
    use tray_icon::{TrayIconBuilder, Icon};

    let icon_image = image::load_from_memory(icon_data)
        .expect("Failed to load icon")
        .to_rgba8();
    let (width, height) = icon_image.dimensions();
    let rgba = icon_image.into_raw();

    let icon = Icon::from_rgba(rgba, width, height).expect("Failed to create tray icon");

    let show_item = MenuItem::new("显示窗口", true, None);
    let quit_item = MenuItem::new("退出", true, None);

    let tray_ids = TrayIds {
        show_id: show_item.id().clone(),
        quit_id: quit_item.id().clone(),
    };

    let tray_menu = Menu::new();
    tray_menu.append(&show_item).expect("Failed to append show menu item");
    tray_menu.append(&quit_item).expect("Failed to append quit menu item");

    let tray = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_menu_on_left_click(false)
        .with_tooltip("Auto Lock - 自动锁屏")
        .with_icon(icon)
        .build()
        .expect("Failed to create tray icon");

    (tray, tray_ids)
}

/// 加载 Windows 系统中文字体，解决 egui 默认字体无法显示中文的问题
///
/// 按优先级尝试加载：微软雅黑 → 黑体 → 宋体，首个成功即停止
fn setup_chinese_fonts(cc: &eframe::CreationContext<'_>) {
    let mut fonts = egui::FontDefinitions::default();

    let font_paths = [
        std::path::PathBuf::from("C:/Windows/Fonts/msyh.ttc"),      // 微软雅黑
        std::path::PathBuf::from("C:/Windows/Fonts/simhei.ttf"),     // 黑体
        std::path::PathBuf::from("C:/Windows/Fonts/simsun.ttc"),     // 宋体
    ];

    for font_path in &font_paths {
        if let Ok(font_data) = std::fs::read(font_path) {
            fonts.font_data.insert(
                "chinese_font".into(),
                Arc::new(egui::FontData::from_owned(font_data)),
            );
            // 插入到字体族首位，确保中文优先使用该字体
            fonts.families.entry(egui::FontFamily::Proportional).or_default()
                .insert(0, "chinese_font".into());
            fonts.families.entry(egui::FontFamily::Monospace).or_default()
                .insert(0, "chinese_font".into());
            break; // 只加载首个可用的字体
        }
    }

    cc.egui_ctx.set_fonts(fonts);
}
