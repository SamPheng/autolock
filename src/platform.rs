use std::process::Command;

// Windows锁屏状态检测
#[cfg(target_os = "windows")]
pub fn is_windows_locked() -> bool {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    use winapi::ctypes::c_void;
    use winapi::um::winuser::{
        CloseDesktop, DESKTOP_READOBJECTS, GetUserObjectInformationW, OpenInputDesktop, UOI_NAME,
    };

    unsafe {
        let hdesk = OpenInputDesktop(0, 0, DESKTOP_READOBJECTS);
        if !hdesk.is_null() {
            let mut desktop_name: [u16; 256] = [0; 256];
            let mut name_len: u32 = 0;
            let result = GetUserObjectInformationW(
                hdesk as *mut c_void,
                UOI_NAME as i32,
                desktop_name.as_mut_ptr() as *mut c_void,
                desktop_name.len() as u32,
                &mut name_len,
            );

            CloseDesktop(hdesk);

            if result != 0 && name_len > 0 {
                let char_count = (name_len / 2) as usize;
                let safe_char_count = char_count.min(256);
                let desktop_name_str = OsString::from_wide(&desktop_name[..safe_char_count])
                    .to_string_lossy()
                    .into_owned();

                if desktop_name_str.contains("Winlogon") {
                    return true;
                }
            }
        }

        return check_session_locked();
    }
}

// 使用会话API检查锁屏状态
#[cfg(target_os = "windows")]
fn check_session_locked() -> bool {
    use winapi::um::winuser::OpenInputDesktop;

    // 方法：尝试打开输入桌面
    // 如果成功打开（未锁屏），返回 false
    // 如果失败（锁屏），返回 true
    unsafe {
        let hdesk = OpenInputDesktop(0, 0, 0);
        if !hdesk.is_null() {
            use winapi::um::winuser::CloseDesktop;
            CloseDesktop(hdesk);
            return false; // 可以打开桌面，未锁屏
        }
        true // 无法打开桌面，已锁屏
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
