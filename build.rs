#[cfg(windows)]
fn main() {
    // 确保图标文件存在
    if !std::path::Path::new("src/icon.ico").exists() {
        panic!("Icon file 'src/icon.ico' not found!");
    }

    winres::WindowsResource::new()
        .set_icon("src/icon.ico") // 设置程序图标（使用ICO格式）
        .set("ProductName", "AutoLock Timer")
        .set("FileDescription", "25分钟自动锁屏计时器")
        .set("OriginalFilename", "autolock.exe")
        .set("CompanyName", "AutoLock")
        .compile()
        .expect("Failed to compile Windows resources");
}

#[cfg(not(windows))]
fn main() {
    // 非Windows平台不执行任何操作
}
