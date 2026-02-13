use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

pub struct Timer {
    duration: Duration,
    start_time: Option<Instant>,
    callback: Option<Box<dyn Fn() + Send + Sync>>,
}

impl Timer {
    pub fn new(minutes: u64) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            duration: Duration::from_secs(minutes * 60),
            start_time: None,
            callback: None,
        }))
    }

    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
    }

    pub fn reset(&mut self) {
        self.start_time = Some(Instant::now());
    }

    pub fn set_callback<F: Fn() + Send + Sync + 'static>(&mut self, callback: F) {
        self.callback = Some(Box::new(callback));
    }

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

    pub fn is_finished(&self) -> bool {
        self.remaining() == Duration::from_secs(0)
    }

    pub fn check_and_trigger(&mut self) {
        if self.is_finished() {
            if let Some(callback) = &self.callback {
                callback();
            }
            self.reset();
        }
    }
}

pub fn start_timer_thread(timer: Arc<Mutex<Timer>>) {
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(1));
            let mut timer = timer.lock().unwrap();
            timer.check_and_trigger();
        }
    });
}
