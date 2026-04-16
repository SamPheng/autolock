use std::thread;
use std::time::Duration;
use winapi::shared::minwindef::{DWORD, FALSE};
use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};
use winapi::um::tlhelp32::{
    CreateToolhelp32Snapshot, PROCESSENTRY32, Process32First, Process32Next,
};
use winapi::um::winuser::{
    FindWindowW, LockWorkStation, SetForegroundWindow, ShowWindow, SW_RESTORE,
};

pub fn trigger_lock() {
    unsafe {
        LockWorkStation();
    }
}

pub fn is_screen_locked() -> bool {
    // 检查LogonUI.exe进程是否在运行，这是Windows登录屏幕的进程
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(0x00000002, 0); // TH32CS_SNAPPROCESS
        if snapshot == INVALID_HANDLE_VALUE {
            return false;
        }

        let mut process_entry: PROCESSENTRY32 = std::mem::zeroed();
        process_entry.dwSize = std::mem::size_of::<PROCESSENTRY32>() as DWORD;

        let mut result = Process32First(snapshot, &mut process_entry);
        let mut found = false;

        while result != FALSE && !found {
            let process_name =
                std::ffi::CStr::from_ptr(process_entry.szExeFile.as_ptr()).to_string_lossy();

            if process_name == "LogonUI.exe" {
                found = true;
            } else {
                result = Process32Next(snapshot, &mut process_entry);
            }
        }

        CloseHandle(snapshot);
        found
    }
}

pub fn monitor_session_events<F: Fn() + Send + 'static>(on_unlock: F) {
    thread::spawn(move || {
        let mut was_locked = false;

        loop {
            thread::sleep(Duration::from_secs(1));

            let is_locked = is_screen_locked();

            // 如果之前被锁定，现在解锁了，触发回调
            if was_locked && !is_locked {
                on_unlock();
            }

            was_locked = is_locked;
        }
    });
}

/// 通过 Win32 API 查找并显示主窗口
pub fn show_main_window() {
    unsafe {
        let title: Vec<u16> = "Auto Lock\0".encode_utf16().collect();
        let hwnd = FindWindowW(std::ptr::null(), title.as_ptr());
        if !hwnd.is_null() {
            ShowWindow(hwnd, SW_RESTORE);
            SetForegroundWindow(hwnd);
        }
    }
}

/// 强制退出程序
pub fn force_exit() {
    std::process::exit(0);
}
