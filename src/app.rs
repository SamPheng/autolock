use crate::timer::Timer;
use crate::SKIP_INTERCEPT;
use eframe::egui;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

pub struct AutolockApp {
    timer: Arc<Mutex<Timer>>,
    #[allow(dead_code)]
    tray: tray_icon::TrayIcon,
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

    fn parsed_minutes(&self) -> u64 {
        self.input_minutes.parse::<u64>().unwrap_or(25).clamp(1, 999)
    }
}

/// 用 painter 测量文本宽度
fn text_width(painter: &egui::Painter, text: &str, font_id: egui::FontId) -> f32 {
    painter.layout_no_wrap(text.to_string(), font_id, egui::Color32::WHITE).size().x
}

impl eframe::App for AutolockApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if SKIP_INTERCEPT.load(Ordering::SeqCst) {
            SKIP_INTERCEPT.store(false, Ordering::SeqCst);
            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
        } else {
            let close_requested = ctx.input(|i| i.viewport().close_requested());
            if close_requested {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
            }
            let minimized = ctx.input(|i| i.viewport().minimized.unwrap_or(false));
            if minimized {
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
            }
        }

        let timer = self.timer.lock().unwrap();
        let is_running = timer.is_running();
        let remaining = timer.remaining();
        let minutes = remaining.as_secs() / 60;
        let seconds = remaining.as_secs() % 60;
        let duration_mins = timer.duration_minutes();
        drop(timer);

        // 预计算文本宽度（在创建 UI 之前，用 ctx 的 painter）
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

        egui::CentralPanel::default().show(ctx, |ui| {
            // 标题
            ui.vertical_centered(|ui| {
                ui.heading(
                    egui::RichText::new("自动锁屏")
                        .size(22.0)
                        .color(egui::Color32::from_rgb(137, 180, 250)), // 柔和蓝
                );
            });
            ui.separator();

            // 倒计时
            ui.vertical_centered(|ui| {
                ui.label(
                    egui::RichText::new(if is_running {
                        format!("{}:{:02}", minutes, seconds)
                    } else {
                        "已停止".to_string()
                    })
                    .size(32.0)
                    .color(if is_running {
                        egui::Color32::from_rgb(166, 227, 161) // 柔和绿
                    } else {
                        egui::Color32::from_rgb(243, 139, 168) // 柔和红
                    }),
                );
            });

            // 进度条
            if is_running {
                let total = duration_mins * 60;
                let elapsed = total - remaining.as_secs();
                let progress = if total > 0 { elapsed as f32 / total as f32 } else { 0.0 };
                ui.vertical_centered(|ui| {
                    ui.add(egui::ProgressBar::new(progress).desired_width(200.0).show_percentage());
                });
            }

            ui.separator();

            // 定时时长输入行
            let avail = ui.available_width();
            let indent = ((avail - input_row_w) / 2.0).max(0.0);
            ui.horizontal(|ui| {
                ui.add_space(indent);
                ui.label(
                    egui::RichText::new("定时时长:").color(egui::Color32::from_rgb(186, 194, 222)), // 浅灰蓝
                );
                let response = ui.add_enabled(
                    !is_running,
                    egui::TextEdit::singleline(&mut self.input_minutes)
                        .desired_width(50.0)
                        .horizontal_align(egui::Align::Center)
                        .text_color(egui::Color32::from_rgb(205, 214, 244)), // 柔和白
                );
                if response.changed() {
                    self.input_minutes.retain(|c| c.is_ascii_digit());
                }
                ui.label(
                    egui::RichText::new("分钟").color(egui::Color32::from_rgb(186, 194, 222)),
                );
            });

            // 操作按钮行
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

        ctx.request_repaint_after(std::time::Duration::from_secs(1));
    }
}
