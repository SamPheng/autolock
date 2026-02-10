#![windows_subsystem = "windows"]
mod app;
mod platform;
mod timer;

use app::LockTimerApp;
use eframe::egui;

fn main() -> Result<(), eframe::Error> {
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

    // ===== 0.33.3 使用新的viewport API =====
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([300.0, 220.0])
            .with_min_inner_size([300.0, 220.0])
            .with_icon(icon),
        ..eframe::NativeOptions::default()
    };

    // 运行应用（0.33.3 官方标准写法）
    eframe::run_native(
        "25分钟锁屏计时器",
        native_options,
        Box::new(|_cc| Ok(Box::new(LockTimerApp::default()))),
    )
}
