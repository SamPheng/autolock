use crate::platform::is_windows_locked;
use crate::timer::TIMER_DURATION_SECONDS;
use eframe::egui;
use std::time::{Duration, Instant};

// 应用状态
pub struct LockTimerApp {
    pub start_time: Instant,     // 计时起点
    pub elapsed_total: u32,      // 累计流逝秒数
    pub remaining_sec: u32,      // 剩余秒数
    pub is_running: bool,        // 是否计时中
    pub is_triggered_lock: bool, // 是否程序触发锁屏
}

impl Default for LockTimerApp {
    fn default() -> Self {
        Self {
            start_time: Instant::now(),
            elapsed_total: 0,
            remaining_sec: TIMER_DURATION_SECONDS,
            is_running: true,
            is_triggered_lock: false,
        }
    }
}

impl eframe::App for LockTimerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 检测系统锁屏状态（Windows）
        #[cfg(target_os = "windows")]
        {
            if self.is_running && !self.is_triggered_lock {
                // 检测锁屏状态
                if is_windows_locked() {
                    // 检测到主动锁屏，停止计时
                    self.is_running = false;
                    self.is_triggered_lock = true;
                }
            }
        }

        // 1. 倒计时核心逻辑（准确无跳变）
        if self.is_running && self.remaining_sec > 0 && !self.is_triggered_lock {
            let total_elapsed = self.start_time.elapsed().as_secs() as u32;
            let new_elapsed = total_elapsed - self.elapsed_total;

            if new_elapsed >= 1 {
                self.remaining_sec = self.remaining_sec.saturating_sub(new_elapsed);
                self.elapsed_total = total_elapsed;

                // 倒计时结束触发锁屏
                if self.remaining_sec == 0 {
                    crate::platform::trigger_lock();
                    self.is_triggered_lock = true;
                    self.is_running = false;
                }
            }
        }

        // 检测解锁状态并自动重启倒计时
        if self.is_triggered_lock && !self.is_running {
            // 等待一段时间后检测解锁状态
            let lock_time_elapsed = self.start_time.elapsed().as_secs() as u32 - self.elapsed_total;
            if lock_time_elapsed > 5 {
                // 检测是否已解锁
                #[cfg(target_os = "windows")]
                {
                    if !is_windows_locked() {
                        // 确认已解锁，自动重启倒计时
                        self.reset_timer();
                    }
                }
                #[cfg(not(target_os = "windows"))]
                {
                    // 非Windows系统，使用简单的时间假设
                    self.reset_timer();
                }
            }
        }

        // 2. 界面绘制（0.33.3 官方 API）
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Lock Timer - 25 min Auto Lock");
            ui.separator();

            // 格式化倒计时为 分:秒
            let minutes = self.remaining_sec / 60;
            let seconds = self.remaining_sec % 60;
            let time_text = format!("{:02}:{:02}", minutes, seconds);

            // 0.33.3 标准写法：使用ui.add方法
            ui.add(egui::Label::new(
                egui::RichText::new(time_text)
                    .size(40.0)
                    .color(egui::Color32::WHITE),
            ));
            ui.separator();

            // 状态显示
            let status_text = if self.is_triggered_lock {
                "Status: Timer ended, screen locked"
            } else if self.is_running {
                "Status: Running (will lock in 25 minutes)"
            } else {
                "Status: Stopped"
            };
            ui.label(status_text);

            // 重置按钮
            if ui.button("Reset Timer").clicked() {
                self.reset_timer();
            }
        });

        // 100ms 刷新界面（官方推荐刷新率）
        ctx.request_repaint_after(Duration::from_millis(100));
    }
}

// 自定义方法：重置倒计时
impl LockTimerApp {
    pub fn reset_timer(&mut self) {
        self.start_time = Instant::now();
        self.elapsed_total = 0;
        self.remaining_sec = TIMER_DURATION_SECONDS;
        self.is_running = true;
        self.is_triggered_lock = false;
    }
}
