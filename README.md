# AutoLock

基于 Rust 和 eframe 的 Windows 自动锁屏工具，界面简洁美观，静默后台运行。

## 功能特性

- **定时锁屏** — 自定义定时时长，到时自动锁定屏幕
- **系统托盘** — 关闭窗口最小化到托盘，双击托盘图标恢复窗口
- **实时倒计时** — 显示剩余时间与进度条，运行中绿色，停止时红色
- **解锁自动重置** — 检测系统解锁事件，自动重置定时器重新计时
- **开机自启** — 启动即开始计时，无需手动操作

## 界面预览

深色 Catppuccin Mocha 风格主题，柔和护眼：

- 标题：柔和蓝
- 倒计时运行中：柔和绿 / 已停止：柔和红
- 按钮/控件：蓝宝石色交互反馈

## 技术栈

| 项目 | 技术 |
|------|------|
| 语言 | Rust (Edition 2024) |
| GUI | eframe 0.33 / egui |
| 托盘 | tray-icon |
| 平台 API | winapi |

## 系统要求

- Windows 10/11
- Rust 1.85+（Edition 2024）

## 安装

### 编译安装

```bash
git clone https://gitee.com/SamPheng/autolock.git
cd autolock
cargo build --release
```

生成的可执行文件位于 `target/release/autolock.exe`。

### 预编译版本

从 [Gitee Releases](https://gitee.com/SamPheng/autolock/releases) 下载。

## 使用方法

1. 启动程序，自动开始 25 分钟倒计时
2. 在输入框中修改定时时长（1~999 分钟），点击「开始」
3. 运行中可点击「停止」暂停计时
4. 关闭窗口后程序最小化到系统托盘继续运行
5. 双击托盘图标或右键「显示窗口」恢复界面
6. 右键托盘图标「退出」关闭程序

## 项目结构

```
autolock/
├── src/
│   ├── main.rs        # 入口：窗口配置、视觉风格、托盘图标、字体加载
│   ├── app.rs         # GUI：倒计时显示、时间输入、操作按钮
│   ├── timer.rs       # 定时器：计时、回调、后台检查线程
│   ├── platform.rs    # 平台：锁屏触发、锁屏检测、会话监听、窗口恢复
│   └── icon.ico       # 应用图标
├── build.rs           # Windows 资源嵌入
└── Cargo.toml
```

## 注意事项

- 仅支持 Windows 平台
- 建议加入开机自启动以获得最佳体验

## 许可证

MIT License
