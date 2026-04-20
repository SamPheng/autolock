//! 应用主界面（eframe::App 实现）
//!
//! 负责：窗口生命周期管理（关闭/最小化拦截、托盘恢复）、定时器 UI、倒计时显示

use crate::timer::Timer;
use crate::SKIP_INTERCEPT;
use crate::platform::set_main_hwnd;
use eframe::egui;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use winapi::shared::windef::HWND;

/// 标记是否已保存过窗口句柄（仅需首帧执行一次）
static HWND_SAVED: AtomicBool = AtomicBool::new(false);

/// 应用主状态
pub struct AutolockApp {
    /// 定时器，跨线程共享（UI 线程 + 定时器后台线程 + 会话监控线程）
    timer: Arc<Mutex<Timer>>,
    /// 托盘图标实例，必须持有以维持其生命周期，drop 后托盘图标消失
    #[allow(dead_code)]
    tray: tray_icon::TrayIcon,
    /// 用户输入的定时时长（分钟），字符串形式以便 TextEdit 直接绑定
    input_minutes: String,
}

impl AutolockApp {
    pub fn new(timer: Arc<Mutex<Timer>>, tray: tray_icon::TrayIcon) -> Self {
        let minutes = timer.lock().unwrap().duration_minutes();
        Self {
            timer,
            tray,
            input_minutes: minutes.to_string(),
        }
    }

    /// 将用户输入解析为分钟数，非法输入回退到 25，范围限定 [1, 999]
    fn parsed_minutes(&self) -> u64 {
        self.input_minutes.parse::<u64>().unwrap_or(25).clamp(1, 999)
    }
}

/// 利用 painter 的文本布局测量文本渲染宽度，用于居中对齐计算
fn text_width(painter: &egui::Painter, text: &str, font_id: egui::FontId) -> f32 {
    painter.layout_no_wrap(text.to_string(), font_id, egui::Color32::WHITE).size().x
}

impl eframe::App for AutolockApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // ── 首帧：保存窗口句柄 ──────────────────────────
        // HWND 供 platform.rs 的 show_main_window() 使用，从后台线程恢复窗口
        if !HWND_SAVED.load(Ordering::SeqCst) {
            if let Ok(handle) = frame.window_handle() {
                if let RawWindowHandle::Win32(win32_handle) = handle.as_raw() {
                    let hwnd = win32_handle.hwnd.get() as HWND;
                    set_main_hwnd(hwnd);
                    HWND_SAVED.store(true, Ordering::SeqCst);
                }
            }
        }

        // ── 拦截关闭按钮：隐藏到托盘而非退出进程 ────────
        let close_requested = ctx.input(|i| i.viewport().close_requested());
        if close_requested {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
        }

        // ── 拦截最小化：隐藏到托盘而非留在任务栏 ────────
        let is_minimized = ctx.input(|i| i.viewport().minimized.unwrap_or(false));
        if is_minimized {
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
        }

        // ── 从托盘恢复窗口 ──────────────────────────────
        // SKIP_INTERCEPT 由后台线程设置，此处消费该标志并恢复窗口可见性
        if SKIP_INTERCEPT.load(Ordering::SeqCst) {
            SKIP_INTERCEPT.store(false, Ordering::SeqCst);
            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
            ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
        }

        // ── 读取定时器状态 ──────────────────────────────
        let timer = self.timer.lock().unwrap();
        let is_running = timer.is_running();
        let remaining = timer.remaining();
        let minutes = remaining.as_secs() / 60;
        let seconds = remaining.as_secs() % 60;
        let duration_mins = timer.duration_minutes();
        drop(timer); // 尽早释放锁，避免在 UI 渲染期间持有

        // ── 预计算布局宽度，用于手动居中对齐 ────────────
        let painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Background, egui::Id::new("measure")));
        let font = egui::FontId::default();
        let btn_font = egui::FontId::proportional(14.0);
        let item_gap = 6.0; // 与 main.rs 中 spacing.item_spacing.x 一致
        let btn_inner = 8.0 * 2.0 + 6.0; // button_padding.x * 2 + 额外边距

        let input_row_w = text_width(&painter, "定时时长:", font.clone())
            + item_gap
            + 50.0 + 8.0  // TextEdit desired_width + 内边距
            + item_gap
            + text_width(&painter, "分钟", font);

        let btn_row_w = text_width(&painter, if is_running { "停止" } else { "开始" }, btn_font.clone()) + btn_inner;

        // ── 渲染 UI ─────────────────────────────────────
        egui::CentralPanel::default().show(ctx, |ui| {
            // 标题
            ui.vertical_centered(|ui| {
                ui.heading(
                    egui::RichText::new("自动锁屏")
                        .size(22.0)
                        .color(egui::Color32::from_rgb(137, 180, 250)), // Blue
                );
            });
            ui.separator();

            // 倒计时显示
            ui.vertical_centered(|ui| {
                ui.label(
                    egui::RichText::new(if is_running {
                        format!("{}:{:02}", minutes, seconds)
                    } else {
                        "已停止".to_string()
                    })
                    .size(32.0)
                    .color(if is_running {
                        egui::Color32::from_rgb(166, 227, 161) // Green
                    } else {
                        egui::Color32::from_rgb(243, 139, 168) // Red
                    }),
                );
            });

            // 进度条（仅运行中显示）
            if is_running {
                let total = duration_mins * 60;
                let elapsed = total - remaining.as_secs();
                let progress = if total > 0 { elapsed as f32 / total as f32 } else { 0.0 };
                ui.vertical_centered(|ui| {
                    ui.add(egui::ProgressBar::new(progress).desired_width(200.0).show_percentage());
                });
            }

            ui.separator();

            // 定时时长输入行（居中对齐）
            let avail = ui.available_width();
            let indent = ((avail - input_row_w) / 2.0).max(0.0);
            ui.horizontal(|ui| {
                ui.add_space(indent);
                ui.label(
                    egui::RichText::new("定时时长:").color(egui::Color32::from_rgb(186, 194, 222)), // Subtext0
                );
                let response = ui.add_enabled(
                    !is_running, // 运行中禁止编辑
                    egui::TextEdit::singleline(&mut self.input_minutes)
                        .desired_width(50.0)
                        .horizontal_align(egui::Align::Center)
                        .text_color(egui::Color32::from_rgb(205, 214, 244)), // Text
                );
                // 只允许输入数字
                if response.changed() {
                    self.input_minutes.retain(|c| c.is_ascii_digit());
                }
                ui.label(
                    egui::RichText::new("分钟").color(egui::Color32::from_rgb(186, 194, 222)),
                );
            });

            // 操作按钮行（居中对齐）
            let avail = ui.available_width();
            let indent = ((avail - btn_row_w) / 2.0).max(0.0);
            ui.horizontal(|ui| {
                ui.add_space(indent);
                if is_running {
                    if ui.button(
                        egui::RichText::new("停止").color(egui::Color32::WHITE),
                    ).clicked() {
                        self.timer.lock().unwrap().stop();
                    }
                } else {
                    if ui.button(
                        egui::RichText::new("开始").color(egui::Color32::WHITE),
                    ).clicked() {
                        let mins = self.parsed_minutes();
                        self.timer.lock().unwrap().set_duration(mins);
                        self.timer.lock().unwrap().start();
                    }
                }
            });
        });

        // 定时刷新 UI，保证倒计时显示实时更新
        ctx.request_repaint_after(std::time::Duration::from_millis(200));
    }
}
