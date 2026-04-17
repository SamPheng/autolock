#[cfg(windows)]
fn main() {
    let icon_path = std::path::Path::new("src/icon.ico");
    if !icon_path.exists() {
        panic!("Icon file 'src/icon.ico' not found!");
    }

    let mut res = winres::WindowsResource::new();
    res.set_icon("src/icon.ico");
    res.set("ProductName", "Auto Lock");
    res.set("FileDescription", "Auto Lock Application");
    res.set("CompanyName", "Auto Lock");
    res.compile().expect("Failed to compile Windows resources");
}

#[cfg(not(windows))]
fn main() {
    // 非Windows平台不执行任何操作
}
