#![windows_subsystem = "windows"]
mod app;
mod platform;
mod timer;

use app::AutolockApp;
use eframe::{NativeOptions, egui};
use platform::{monitor_session_events, force_exit, show_main_window, trigger_lock};
use std::sync::atomic::{AtomicBool, Ordering};
use timer::{Timer, start_timer_thread};
use tray_icon::menu::{Menu, MenuId, MenuItem};
use tray_icon::TrayIconEvent;

pub struct TrayIds {
    pub show_id: MenuId,
    pub quit_id: MenuId,
}

/// 全局标志：从托盘恢复窗口后，跳过几帧拦截逻辑
/// 防止窗口恢复后 eframe 内部 minimized 状态残留导致立即被重新隐藏
pub static SKIP_INTERCEPT: AtomicBool = AtomicBool::new(false);

fn main() {
    // 加载图标数据
    let icon_data = include_bytes!("icon.ico");
    let icon_image = image::load_from_memory(icon_data)
        .expect("Failed to load icon")
        .to_rgba8();
    let (width, height) = icon_image.dimensions();

    // 创建egui图标
    let icon = egui::IconData {
        rgba: icon_image.into_raw(),
        width,
        height,
    };

    // 创建定时器
    let timer = Timer::new(25);

    {
        timer.lock().unwrap().set_callback(move || {
            trigger_lock();
        });
    }

    start_timer_thread(timer.clone());

    {
        let timer_clone = timer.clone();
        monitor_session_events(move || {
            timer_clone.lock().unwrap().reset();
        });
    }

    timer.lock().unwrap().start();

    // 保存图标数据，用于创建托盘图标
    let tray_icon_data = icon_data.to_vec();

    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([300.0, 220.0])
            .with_min_inner_size([300.0, 220.0])
            .with_icon(icon),
        ..Default::default()
    };

    let _ = eframe::run_native(
        "Auto Lock",
        options,
        Box::new(move |_cc| {
            // 在 eframe 创建回调中创建托盘图标
            let (tray, tray_ids) = create_tray_icon(&tray_icon_data);

            // 启动后台线程：监听菜单事件和托盘图标双击事件
            let show_id = tray_ids.show_id.clone();
            let quit_id = tray_ids.quit_id.clone();
            std::thread::spawn(move || {
                loop {
                    // 监听菜单事件
                    if let Ok(event) = tray_icon::menu::MenuEvent::receiver().try_recv() {
                        if event.id == show_id {
                            crate::SKIP_INTERCEPT.store(true, Ordering::SeqCst);
                            show_main_window();
                        } else if event.id == quit_id {
                            force_exit();
                        }
                    }

                    // 监听托盘图标事件（双击显示窗口）
                    if let Ok(event) = TrayIconEvent::receiver().try_recv() {
                        if let TrayIconEvent::DoubleClick { .. } = event {
                            crate::SKIP_INTERCEPT.store(true, Ordering::SeqCst);
                            show_main_window();
                        }
                    }

                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
            });

            Ok(Box::new(AutolockApp::new(timer, tray)))
        }),
    );
}

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
