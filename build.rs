#[cfg(windows)]
fn main() {
    println!("Building Windows resources...");

    // 确保图标文件存在（使用相对路径）
    let icon_path = std::path::Path::new("src/icon.ico");
    println!("Checking icon file: {:?}", icon_path);
    if !icon_path.exists() {
        panic!("Icon file 'src/icon.ico' not found!");
    }
    println!("Icon file found: {:?}", icon_path);

    // 使用winres设置图标和其他资源
    println!("Setting icon for Windows resources...");
    let mut res = winres::WindowsResource::new();
    res.set_icon("src/icon.ico");
    res.set("ProductName", "Auto Lock");
    res.set("FileDescription", "Auto Lock Application");
    res.set("CompanyName", "Auto Lock");
    res.compile().expect("Failed to compile Windows resources");

    println!("Windows resources compiled successfully!");
}

#[cfg(not(windows))]
fn main() {
    // 非Windows平台不执行任何操作
}
