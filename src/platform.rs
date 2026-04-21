//! Windows 平台相关功能
//!
//! 提供：屏幕锁定/解锁检测、工作站锁定、单实例互斥体、
//! 窗口句柄管理、命名事件进程间通信、窗口显示/恢复

use std::thread;
use std::time::Duration;
use winapi::shared::minwindef::{DWORD, FALSE};
use winapi::shared::windef::HWND;
use winapi::shared::winerror::ERROR_ALREADY_EXISTS;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};
use winapi::um::synchapi::{CreateEventW, ResetEvent, SetEvent, WaitForSingleObject};
use winapi::um::tlhelp32::{
    CreateToolhelp32Snapshot, PROCESSENTRY32, Process32First, Process32Next,
};
use winapi::um::winbase::INFINITE;
use winapi::um::winuser::{
    GetWindowLongW, IsIconic, LockWorkStation, SetForegroundWindow, SetWindowLongW, ShowWindow,
    GWL_EXSTYLE, SW_RESTORE, SW_SHOW, WS_EX_APPWINDOW, WS_EX_TOOLWINDOW,
};

/// 全局存储主窗口句柄，由 `app.rs` 首帧通过 `set_main_hwnd()` 设置
static MAIN_HWND: std::sync::atomic::AtomicPtr<std::ffi::c_void> =
    std::sync::atomic::AtomicPtr::new(std::ptr::null_mut());

/// 保存主窗口 HWND（由 eframe 首帧调用）
pub fn set_main_hwnd(hwnd: HWND) {
    MAIN_HWND.store(hwnd as *mut _, std::sync::atomic::Ordering::SeqCst);
}

/// 调用 Win32 API 锁定工作站（等同 Win+L）
pub fn trigger_lock() {
    unsafe {
        LockWorkStation();
    }
}

/// 检测屏幕是否处于锁定状态
///
/// 通过遍历进程快照查找 `LogonUI.exe`（Windows 登录屏幕进程）来判断
fn is_screen_locked() -> bool {
    unsafe {
        // TH32CS_SNAPPROCESS = 0x00000002
        let snapshot = CreateToolhelp32Snapshot(0x00000002, 0);
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

/// 启动后台线程监控屏幕锁定/解锁状态
///
/// 检测到屏幕从锁定变为解锁时，调用 `on_unlock` 回调（用于重置定时器）
pub fn monitor_session_events<F: Fn() + Send + 'static>(on_unlock: F) {
    thread::spawn(move || {
        let mut was_locked = false;

        loop {
            thread::sleep(Duration::from_secs(1));
            let is_locked = is_screen_locked();

            // 锁定 → 解锁：触发回调
            if was_locked && !is_locked {
                on_unlock();
            }

            was_locked = is_locked;
        }
    });
}

/// 强制退出整个进程
pub fn force_exit() {
    std::process::exit(0);
}

/// 尝试获取单实例命名互斥体
///
/// - `Ok(())`：当前是首个实例
/// - `Err(())`：已有实例运行，互斥体已存在
pub fn try_single_instance() -> Result<(), ()> {
    unsafe {
        let name: Vec<u16> = "AutoLock_SingleInstance\0".encode_utf16().collect();
        let handle = winapi::um::synchapi::CreateMutexW(
            std::ptr::null_mut(),
            0,
            name.as_ptr(),
        );
        if GetLastError() == ERROR_ALREADY_EXISTS {
            if !handle.is_null() {
                CloseHandle(handle);
            }
            Err(())
        } else {
            Ok(())
        }
    }
}

/// 通知已有实例显示窗口
///
/// 通过命名事件 `AutoLock_ShowWindow` 发送信号，已运行的实例监听此事件后恢复窗口
pub fn notify_existing_instance() {
    unsafe {
        let name: Vec<u16> = "AutoLock_ShowWindow\0".encode_utf16().collect();
        let event = CreateEventW(
            std::ptr::null_mut(),
            1, // manual reset
            0, // initial state: nonsignaled
            name.as_ptr(),
        );
        if !event.is_null() {
            SetEvent(event);
            CloseHandle(event);
        }
    }
}

/// 从任务栏隐藏窗口（最小化到托盘时调用）
///
/// 通过修改扩展窗口样式：移除 `WS_EX_APPWINDOW`，添加 `WS_EX_TOOLWINDOW`，
/// 使窗口不出现在任务栏中。配合 eframe 的 Minimized(true) 使用，
/// eframe 内部的 is_minimized 检测会生效并执行 sleep(10ms)，降低 CPU 占用。
pub fn hide_from_taskbar() {
    let hwnd = MAIN_HWND.load(std::sync::atomic::Ordering::SeqCst) as HWND;
    if hwnd.is_null() {
        return;
    }
    unsafe {
        set_taskbar_visible(hwnd, false);
    }
}

/// 通过 Win32 API 恢复并聚焦主窗口
///
/// 从后台线程直接操作，不依赖 eframe update 循环。
/// 处理两种情况：窗口最小化（IsIconic → SW_RESTORE）和窗口隐藏（SW_SHOW）
pub fn show_main_window() {
    let hwnd = MAIN_HWND.load(std::sync::atomic::Ordering::SeqCst) as HWND;
    if hwnd.is_null() {
        return;
    }
    unsafe {
        set_taskbar_visible(hwnd, true);
        if IsIconic(hwnd) != 0 {
            ShowWindow(hwnd, SW_RESTORE); // 先从最小化恢复
        }
        ShowWindow(hwnd, SW_SHOW); // 显示窗口
        SetForegroundWindow(hwnd); // 提升到前台
    }
}

/// 设置窗口是否在任务栏中显示
///
/// - `visible = true`：正常显示在任务栏（WS_EX_APPWINDOW，无 WS_EX_TOOLWINDOW）
/// - `visible = false`：从任务栏隐藏（WS_EX_TOOLWINDOW，无 WS_EX_APPWINDOW）
unsafe fn set_taskbar_visible(hwnd: HWND, visible: bool) {
    unsafe {
        let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE) as u32;
        if visible {
            SetWindowLongW(hwnd, GWL_EXSTYLE, ((ex_style | WS_EX_APPWINDOW) & !WS_EX_TOOLWINDOW) as i32);
        } else {
            SetWindowLongW(hwnd, GWL_EXSTYLE, ((ex_style | WS_EX_TOOLWINDOW) & !WS_EX_APPWINDOW) as i32);
        }
    }
}

/// 启动后台线程监听命名事件 `AutoLock_ShowWindow`
///
/// 新实例启动时会通过 `notify_existing_instance()` 触发此事件，
/// 收到信号后调用 `on_show` 回调恢复窗口
pub fn listen_show_window<F: Fn() + Send + 'static>(on_show: F) {
    thread::spawn(move || unsafe {
        let name: Vec<u16> = "AutoLock_ShowWindow\0".encode_utf16().collect();
        let event = CreateEventW(
            std::ptr::null_mut(),
            1, // manual reset
            0, // initial state: nonsignaled
            name.as_ptr(),
        );
        if event.is_null() {
            return;
        }
        loop {
            WaitForSingleObject(event, INFINITE); // 阻塞等待信号
            on_show();
            ResetEvent(event); // 手动重置，为下次监听做准备
        }
    });
}
