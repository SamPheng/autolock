use std::process::Command;

// Windows锁屏状态检测 - 使用更简单可靠的方法
#[cfg(target_os = "windows")]
pub fn is_windows_locked() -> bool {
    use winapi::um::winuser::{GetForegroundWindow, GetWindowThreadProcessId};

    unsafe {
        // 获取前台窗口
        let hwnd = GetForegroundWindow();
        if hwnd.is_null() {
            return true; // 无前台窗口，可能已锁屏
        }

        // 获取拥有前台窗口的进程ID
        let mut process_id: u32 = 0;
        GetWindowThreadProcessId(hwnd, &mut process_id as *mut u32);

        // 如果进程ID为0或不存在，可能已锁屏
        if process_id == 0 {
            return true;
        }

        false
    }
}

// 跨平台锁屏（Windows 优先）
pub fn trigger_lock() {
    #[cfg(target_os = "windows")]
    {
        match Command::new("rundll32")
            .args(["user32.dll", "LockWorkStation"])
            .status()
        {
            Ok(_) => {}
            Err(e) => {
                // 使用Windows错误对话框显示错误信息
                let _ = msgbox::create(
                    "锁屏失败",
                    &format!("Windows 锁屏命令执行失败：{}", e),
                    msgbox::IconType::Error,
                );
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        let _ = Command::new("pmset").arg("displaysleepnow").status();
        let _ = Command::new("osascript")
            .args(["-e", "tell application \"System Events\" to keystroke \"q\" using {command down, control down}"])
            .status();
    }

    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("loginctl").arg("lock-session").status();
        let _ = Command::new("xdg-screensaver").arg("lock").status();
    }
}
