use std::process::Command;

// Windows锁屏状态检测 - 使用改进的方法
#[cfg(target_os = "windows")]
pub fn is_windows_locked() -> bool {
    use winapi::shared::minwindef::DWORD;
    use winapi::um::winuser::{GetForegroundWindow, GetWindowThreadProcessId};

    unsafe {
        // 获取前台窗口
        let hwnd = GetForegroundWindow();

        // 如果没有前台窗口，UAC弹窗时也会出现这种情况
        // 所以我们需要额外检查
        if hwnd.is_null() {
            // 检查explorer.exe是否在运行 - 锁屏时explorer通常仍在运行
            // 但UAC弹窗时explorer也在运行，所以这个检查主要是排除系统崩溃的情况

            // 简单起见，我们认为如果没有前台窗口但系统仍在运行，就不算锁屏
            // 这样可以避免UAC弹窗时的误判
            return false;
        }

        // 获取拥有前台窗口的进程ID
        let mut process_id: DWORD = 0;
        GetWindowThreadProcessId(hwnd, &mut process_id as *mut DWORD);

        // 如果进程ID为0，可能已锁屏
        if process_id == 0 {
            return true;
        }

        // 未检测到锁屏状态
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
