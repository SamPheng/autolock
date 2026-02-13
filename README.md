

# AutoLock

基于 Rust 和 eframe 框架开发的 Windows 桌面自动锁屏工具。

## 功能特性

- **自动锁屏**：设置空闲时间后，系统将自动锁定屏幕
- **托盘运行**：程序最小化至系统托盘，不影响日常工作
- **实时显示**：界面显示剩余时间和进度条
- **灵活设置**：可随时调整定时时间
- **事件响应**：检测系统解锁状态，智能管理计时器

## 技术栈

- **语言**：Rust
- **GUI 框架**：eframe / egui
- **平台**：Windows

## 系统要求

- Windows 10/11
- Rust 1.60+

## 安装说明

### 方式一：编译安装

```bash
# 克隆项目
git clone https://gitee.com/SamPheng/autolock.git
cd autolock

# 编译发布版本
cargo build --release

# 运行程序
cargo run --release
```

### 方式二：使用预编译版本

从 [Gitee releases](https://gitee.com/SamPheng/autolock/releases) 下载预编译的 `autolock.exe`。

## 使用方法

1. 启动程序后，点击输入框设置锁屏时间（分钟）
2. 点击「开始」按钮启动定时器
3. 程序将显示剩余时间和进度条
4. 到达设定时间后自动锁定屏幕
5. 可随时点击「重置」重新开始计时

## 项目结构

```
autolock/
├── src/
│   ├── main.rs        # 程序入口，Windows 子系统配置
│   ├── app.rs         # GUI 应用主逻辑
│   ├── timer.rs       # 定时器核心实现
│   ├── platform.rs    # Windows 平台特定功能
│   ├── icon.ico       # 应用图标
│   └── icon.svg       # SVG 图标
├── build.rs           # 构建脚本
└── Cargo.toml         # 项目配置
```

## 核心模块

### Timer (timer.rs)

定时器核心组件，提供：
- 计时管理
- 回调函数设置
- 剩余时间查询
- 完成状态检测

### AutolockApp (app.rs)

egui GUI 应用：
- 状态显示
- 参数设置
- 交互控制

### Platform (platform.rs)

Windows 平台适配：
- 触发锁屏 (`trigger_lock`)
- 检测锁屏状态 (`is_screen_locked`)
- 监听会话事件 (`monitor_session_events`)

## 注意事项

- 本程序专为 Windows 平台设计
- 需要管理员权限以实现自动锁屏功能
- 建议将程序加入开机自启动以获得最佳体验

## 许可证

本项目遵循 MIT 许可证。