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
            ui.heading("Auto Lock");
            ui.separator();

            let remaining = self.timer.lock().unwrap().remaining();
            let minutes = remaining.as_secs() / 60;
            let seconds = remaining.as_secs() % 60;

            ui.horizontal(|ui| {
                ui.label("Time left:");
                ui.monospace(format!("{}:{:02}", minutes, seconds));
            });

            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Reset Timer").clicked() {
                    self.timer.lock().unwrap().reset();
                }
            });

            ui.label("Auto lock every 25 minutes");
            ui.label("Restart timer after unlock");
            ui.label("Restart timer after manual lock and unlock");
        });

        ctx.request_repaint_after(std::time::Duration::from_secs(1));
    }
}
