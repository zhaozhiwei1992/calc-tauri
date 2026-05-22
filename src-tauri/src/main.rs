// 防止 Windows 发布版出现额外的控制台窗口，勿删！
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Tauri v2 入口：调用 lib.rs 中定义的 run() 函数
// 这样桌面端和移动端共享同一个入口逻辑
fn main() {
    app_lib::run();
}
