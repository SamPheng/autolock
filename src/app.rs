use crate::timer::Timer;
use crate::SKIP_INTERCEPT;
use eframe::egui;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

pub struct AutolockApp {
    timer: Arc<Mutex<Timer>>,
    #[allow(dead_code)]
    tray: tray_icon::TrayIcon,
}

impl AutolockApp {
    pub fn new(timer: Arc<Mutex<Timer>>, tray: tray_icon::TrayIcon) -> Self {
        Self { timer, tray }
    }
}

impl eframe::App for AutolockApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 如果刚从托盘恢复，跳过拦截逻辑，避免 minimized 状态残留导致立即重新隐藏
        if SKIP_INTERCEPT.load(Ordering::SeqCst) {
            SKIP_INTERCEPT.store(false, Ordering::SeqCst);
            // 主动清除 minimized 状态
            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
        } else {
            // 拦截关闭请求，隐藏到托盘
            let close_requested = ctx.input(|i| i.viewport().close_requested());
            if close_requested {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
            }

            // 拦截最小化，隐藏到托盘
            let minimized = ctx.input(|i| i.viewport().minimized.unwrap_or(false));
            if minimized {
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            let available_height = ui.available_height();

            // 添加上方空间，使内容垂直居中
            ui.add_space(available_height / 2.0 - 80.0);

            ui.vertical_centered(|ui| {
                ui.heading("Auto Lock");
                ui.separator();

                let remaining = self.timer.lock().unwrap().remaining();
                let minutes = remaining.as_secs() / 60;
                let seconds = remaining.as_secs() % 60;

                ui.add_space(10.0);
                ui.label(egui::RichText::new("Time left:").size(20.0));
                ui.label(
                    egui::RichText::new(format!("{}:{:02}", minutes, seconds))
                        .size(30.0)
                        .color(egui::Color32::from_rgb(255, 140, 0)),
                );
                ui.add_space(10.0);

                ui.separator();
            });
        });

        ctx.request_repaint_after(std::time::Duration::from_secs(1));
    }
}
