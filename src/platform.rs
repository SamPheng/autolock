use std::thread;
use std::time::Duration;
use windows_sys::Win32::Foundation::{
    CloseHandle, HWND, INVALID_HANDLE_VALUE, TRUE,
};
use windows_sys::Win32::System::Threading::{
    ResetEvent, SetEvent, WaitForSingleObject, INFINITE,
};

type CreateEventWFn = unsafe extern "system" fn(*mut std::ffi::c_void, i32, i32, *const u16) -> *mut std::ffi::c_void;
type CreateMutexWFn = unsafe extern "system" fn(*mut std::ffi::c_void, i32, *const u16) -> *mut std::ffi::c_void;
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32First, Process32Next, PROCESSENTRY32, TH32CS_SNAPPROCESS,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GetWindowLongPtrW, IsIconic, SetForegroundWindow, SetWindowLongPtrW, ShowWindow, GWL_EXSTYLE,
    SW_RESTORE, SW_SHOW, WS_EX_APPWINDOW, WS_EX_TOOLWINDOW,
};

static MAIN_HWND: std::sync::atomic::AtomicPtr<std::ffi::c_void> =
    std::sync::atomic::AtomicPtr::new(std::ptr::null_mut());

pub fn set_main_hwnd(hwnd: HWND) {
    MAIN_HWND.store(hwnd as *mut _, std::sync::atomic::Ordering::SeqCst);
}

pub fn trigger_lock() {
    use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};
    unsafe {
        let user32 = GetModuleHandleA(b"user32.dll\0".as_ptr());
        if user32 != std::ptr::null_mut() {
            if let Some(lock_work_station) = GetProcAddress(user32, b"LockWorkStation\0".as_ptr()) {
                let func: unsafe extern "system" fn() -> isize = lock_work_station;
                let _ = func();
            }
        }
    }
}

fn is_screen_locked() -> bool {
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot == INVALID_HANDLE_VALUE {
            return false;
        }

        let mut process_entry: PROCESSENTRY32 = std::mem::zeroed();
        process_entry.dwSize = std::mem::size_of::<PROCESSENTRY32>() as u32;

        let result = Process32First(snapshot, &mut process_entry);
        if result != TRUE {
            let _ = CloseHandle(snapshot);
            return false;
        }

        let mut found = false;
        loop {
            let process_name = std::ffi::CStr::from_ptr(process_entry.szExeFile.as_ptr())
                .to_string_lossy();

            if process_name == "LogonUI.exe" {
                found = true;
                break;
            }

            let result = Process32Next(snapshot, &mut process_entry);
            if result != TRUE {
                break;
            }
        }

        let _ = CloseHandle(snapshot);
        found
    }
}

pub fn monitor_session_events<F: Fn() + Send + 'static>(on_unlock: F) {
    thread::spawn(move || {
        let mut was_locked = false;

        loop {
            thread::sleep(Duration::from_secs(1));
            let is_locked = is_screen_locked();

            if was_locked && !is_locked {
                on_unlock();
            }

            was_locked = is_locked;
        }
    });
}

pub fn force_exit() {
    std::process::exit(0);
}

pub fn try_single_instance() -> Result<(), ()> {
    unsafe {
        let kernel32 = windows_sys::Win32::System::LibraryLoader::GetModuleHandleA(b"kernel32.dll\0".as_ptr());
        if kernel32 == std::ptr::null_mut() {
            return Err(());
        }
        let create_mutex_w = windows_sys::Win32::System::LibraryLoader::GetProcAddress(kernel32, b"CreateMutexW\0".as_ptr());
        if create_mutex_w.is_none() {
            return Err(());
        }
        let create_mutex_w: CreateMutexWFn = std::mem::transmute(create_mutex_w.unwrap());

        let name: Vec<u16> = "AutoLock_SingleInstance".encode_utf16().chain(std::iter::once(0)).collect();
        let handle = create_mutex_w(std::ptr::null_mut(), 0, name.as_ptr());
        if handle == std::ptr::null_mut() {
            return Err(());
        }

        let last_error = windows_sys::Win32::Foundation::GetLastError();
        if last_error == windows_sys::Win32::Foundation::ERROR_ALREADY_EXISTS {
            let _ = CloseHandle(handle);
            Err(())
        } else {
            Ok(())
        }
    }
}

pub fn notify_existing_instance() {
    unsafe {
        let kernel32 = windows_sys::Win32::System::LibraryLoader::GetModuleHandleA(b"kernel32.dll\0".as_ptr());
        if kernel32 == std::ptr::null_mut() {
            return;
        }
        let create_event_w = windows_sys::Win32::System::LibraryLoader::GetProcAddress(kernel32, b"CreateEventW\0".as_ptr());
        if create_event_w.is_none() {
            return;
        }
        let create_event_w: CreateEventWFn = std::mem::transmute(create_event_w.unwrap());

        let name: Vec<u16> = "AutoLock_ShowWindow".encode_utf16().chain(std::iter::once(0)).collect();
        let event = create_event_w(std::ptr::null_mut(), TRUE, 0, name.as_ptr());
        if event != std::ptr::null_mut() {
            let _ = SetEvent(event);
            let _ = CloseHandle(event);
        }
    }
}

pub fn hide_from_taskbar() {
    let hwnd = MAIN_HWND.load(std::sync::atomic::Ordering::SeqCst);
    if hwnd.is_null() {
        return;
    }
    unsafe {
        set_taskbar_visible(hwnd as HWND, false);
    }
}

pub fn show_main_window() {
    let hwnd = MAIN_HWND.load(std::sync::atomic::Ordering::SeqCst);
    if hwnd.is_null() {
        return;
    }
    unsafe {
        let hwnd = hwnd as HWND;
        set_taskbar_visible(hwnd, true);
        if IsIconic(hwnd) != 0 {
            let _ = ShowWindow(hwnd, SW_RESTORE);
        }
        let _ = ShowWindow(hwnd, SW_SHOW);
        let _ = SetForegroundWindow(hwnd);
    }
}

unsafe fn set_taskbar_visible(hwnd: HWND, visible: bool) {
    let ex_style = unsafe { GetWindowLongPtrW(hwnd, GWL_EXSTYLE) } as u32;
    if visible {
        unsafe {
            let _ = SetWindowLongPtrW(
                hwnd,
                GWL_EXSTYLE,
                ((ex_style | WS_EX_APPWINDOW) & !WS_EX_TOOLWINDOW) as _,
            );
        }
    } else {
        unsafe {
            let _ = SetWindowLongPtrW(
                hwnd,
                GWL_EXSTYLE,
                ((ex_style | WS_EX_TOOLWINDOW) & !WS_EX_APPWINDOW) as _,
            );
        }
    }
}

pub fn listen_show_window<F: Fn() + Send + 'static>(on_show: F) {
    thread::spawn(move || unsafe {
        let kernel32 = windows_sys::Win32::System::LibraryLoader::GetModuleHandleA(b"kernel32.dll\0".as_ptr());
        if kernel32 == std::ptr::null_mut() {
            return;
        }
        let create_event_w = windows_sys::Win32::System::LibraryLoader::GetProcAddress(kernel32, b"CreateEventW\0".as_ptr());
        if create_event_w.is_none() {
            return;
        }
        let create_event_w: CreateEventWFn = std::mem::transmute(create_event_w.unwrap());

        let name: Vec<u16> = "AutoLock_ShowWindow".encode_utf16().chain(std::iter::once(0)).collect();
        let event = create_event_w(std::ptr::null_mut(), TRUE, 0, name.as_ptr());
        if event != std::ptr::null_mut() {
            loop {
                let _ = WaitForSingleObject(event, INFINITE);
                on_show();
                let _ = ResetEvent(event);
            }
        }
    });
}
