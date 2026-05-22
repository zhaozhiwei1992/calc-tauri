// 声明 calc 子模块
mod calc;

use calc::basic::BasicCalculator;
use calc::{CalcRequest, CalcResponse, Calculator};
use std::sync::Mutex;

/// Tauri 命令：处理计算器按钮动作
///
/// # 参数
/// - `state`: 通过 tauri::State 注入的全局计算器实例
///   Tauri 自动从 manage() 注册的状态中获取
/// - `req`: 前端发送的按钮动作请求
///
/// # 返回
/// CalcResponse 包含新的显示内容和可能的错误信息
///
/// # 前端调用方式
/// ```typescript
/// import { invoke } from '@tauri-apps/api/core'
/// const result = await invoke('calculate', { req: { action: 'number', value: '5' } })
/// ```
///
/// # 关于 Mutex
/// tauri::State 本身不是线程安全的，需要用 Mutex 包裹。
/// Mutex 保证同一时刻只有一个线程能访问计算器状态。
/// lock().unwrap() 在正常情况下不会 panic。
#[tauri::command]
fn calculate(state: tauri::State<Mutex<BasicCalculator>>, req: CalcRequest) -> CalcResponse {
    let mut calculator = state.lock().unwrap();
    calculator.handle_action(req)
}

/// Tauri 命令：重置计算器
///
/// 独立于 calculate 命令，提供显式的重置操作。
/// 返回重置后的初始状态。
#[tauri::command]
fn reset_calculator(state: tauri::State<Mutex<BasicCalculator>>) -> CalcResponse {
    let mut calculator = state.lock().unwrap();
    calculator.reset();
    CalcResponse {
        display: String::from("0"),
        expression: String::new(),
        error: None,
    }
}

/// 应用入口（Tauri v2 格式）
///
/// Tauri v2 将入口函数改为 pub fn run()，放在 lib.rs 中，
/// 由 main.rs 调用。这样桌面端和移动端可以共享同一份代码。
///
/// 启动流程：
/// 1. Builder::default() 创建 Tauri 构建器
/// 2. .manage() 注册全局状态（BasicCalculator 实例）
/// 3. .invoke_handler() 注册前端可调用的命令
/// 4. .run() 启动事件循环
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // 注册全局状态：BasicCalculator 被 Mutex 包裹以保证线程安全
        // manage() 会在应用生命周期内持有这个状态，
        // 命令函数通过 tauri::State 参数获取引用
        .manage(Mutex::new(BasicCalculator::new()))
        // 注册 Tauri 命令，供前端通过 invoke() 调用
        // generate_handler! 宏会在编译时生成类型安全的调用代码
        .invoke_handler(tauri::generate_handler![calculate, reset_calculator])
        .run(tauri::generate_context!())
        .expect("启动 Tauri 应用失败");
}
