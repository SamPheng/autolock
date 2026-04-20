//! 倒计时定时器
//!
//! 提供：启动/停止/重置定时器、设置时长与回调、到期自动触发回调。
//! 通过 `Arc<Mutex<Self>>` 在 UI 线程与后台线程间共享。

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// 倒计时定时器
///
/// 基于 `Instant` 记录起始时间，计算剩余时长。到期时调用预设回调。
/// `new()` 返回 `Arc<Mutex<Self>>` 以便跨线程共享。
pub struct Timer {
    /// 定时时长
    duration: Duration,
    /// 倒计时起始时刻，`None` 表示未启动
    start_time: Option<Instant>,
    /// 到期回调
    callback: Option<Box<dyn Fn() + Send + Sync>>,
}

impl Timer {
    /// 创建定时器并包装为 `Arc<Mutex<Self>>`
    ///
    /// `minutes` 为初始定时时长（分钟）
    pub fn new(minutes: u64) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            duration: Duration::from_secs(minutes * 60),
            start_time: None,
            callback: None,
        }))
    }

    /// 启动定时器，从当前时刻开始倒计时
    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
    }

    /// 停止定时器，清除起始时刻
    pub fn stop(&mut self) {
        self.start_time = None;
    }

    /// 重置定时器，从当前时刻重新开始倒计时
    pub fn reset(&mut self) {
        self.start_time = Some(Instant::now());
    }

    /// 设置定时时长并停止计时
    pub fn set_duration(&mut self, minutes: u64) {
        self.duration = Duration::from_secs(minutes * 60);
        self.start_time = None;
    }

    /// 设置到期回调
    pub fn set_callback<F: Fn() + Send + Sync + 'static>(&mut self, callback: F) {
        self.callback = Some(Box::new(callback));
    }

    /// 计算剩余时长
    ///
    /// - 运行中：`duration - elapsed`（最小为 0）
    /// - 未启动：返回完整 `duration`
    pub fn remaining(&self) -> Duration {
        if let Some(start_time) = self.start_time {
            let elapsed = start_time.elapsed();
            if elapsed >= self.duration {
                Duration::from_secs(0)
            } else {
                self.duration - elapsed
            }
        } else {
            self.duration
        }
    }

    /// 定时器是否正在运行
    pub fn is_running(&self) -> bool {
        self.start_time.is_some()
    }

    /// 获取定时时长（分钟）
    pub fn duration_minutes(&self) -> u64 {
        self.duration.as_secs() / 60
    }

    /// 检查是否到期，若到期则触发回调并停止计时
    pub fn check_and_trigger(&mut self) {
        if self.start_time.is_none() {
            return;
        }
        if self.remaining() == Duration::from_secs(0) {
            if let Some(callback) = &self.callback {
                callback();
            }
            self.start_time = None;
        }
    }
}

/// 启动后台线程，每秒检查定时器是否到期
pub fn start_timer_thread(timer: Arc<Mutex<Timer>>) {
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(1));
            let mut timer = timer.lock().unwrap();
            timer.check_and_trigger();
        }
    });
}
