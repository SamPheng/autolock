#![windows_subsystem = "windows"]
mod app;
mod platform;
mod timer;

use app::AutolockApp;
use eframe::{NativeOptions, egui};
use platform::{monitor_session_events, trigger_lock};
use timer::{Timer, start_timer_thread};

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
        Box::new(|_cc| Ok(Box::new(AutolockApp::new(timer)))),
    );
}
