use crate::timer::Timer;
use eframe::egui;
use std::sync::{Arc, Mutex};

pub struct AutolockApp {
    timer: Arc<Mutex<Timer>>,
}

impl AutolockApp {
    pub fn new(timer: Arc<Mutex<Timer>>) -> Self {
        Self { timer }
    }
}

impl eframe::App for AutolockApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
