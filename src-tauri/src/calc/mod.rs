// 计算器模块入口
// 提供计算器 trait 定义、数据结构和子模块管理
//
// 模块结构:
//   mod.rs     - 本文件，定义公共接口和数据结构
//   basic.rs   - 基础四则运算计算器实现
//   error.rs   - 自定义错误类型
//
// 设计理念:
//   通过 Calculator trait 抽象计算器行为，
//   BasicCalculator 实现当前的基础四则运算，
//   未来添加 ScientificCalculator 只需实现同一 trait

pub mod basic;
pub mod error;

use serde::{Deserialize, Serialize};

/// 前端发来的按钮动作请求
///
/// 每次用户点击按钮，前端只发送一个 action + value。
/// Rust 后端通过 tauri::State 维护计算器内部状态，
/// 不需要前端传递完整状态。
///
/// # 示例
/// ```
/// // 用户点击数字键 "5"
/// CalcRequest { action: "number", value: "5" }
///
/// // 用户点击加号
/// CalcRequest { action: "operator", value: "+" }
///
/// // 用户点击等号
/// CalcRequest { action: "equals", value: "" }
/// ```
#[derive(Debug, Deserialize)]
pub struct CalcRequest {
    /// 动作类型:
    /// - "number": 数字输入 (0-9)
    /// - "operator": 操作符 (+, -, ×, ÷)
    /// - "equals": 等号（执行运算）
    /// - "clear": 清除（归零）
    /// - "percent": 百分比
    /// - "negate": 正负号切换
    /// - "decimal": 小数点
    pub action: String,

    /// 具体值:
    /// - 数字键传 "0"-"9"
    /// - 操作符传 "+"/"-"/"×"/"÷"
    /// - 其他动作可为空字符串
    pub value: String,
}

/// 返回给前端的计算结果
///
/// 前端根据此结构更新显示屏。
/// error 字段非空时，前端显示错误提示。
#[derive(Debug, Serialize)]
pub struct CalcResponse {
    /// 显示屏主数字（如 "15"）
    pub display: String,

    /// 表达式行（如 "12 + 3 ="），显示历史运算
    pub expression: String,

    /// 错误信息（除零、溢出等），无错误时为 None
    pub error: Option<String>,
}

/// 计算器 trait - 所有计算器类型的统一接口
///
/// BasicCalculator 和未来的 ScientificCalculator 都实现此 trait。
/// 在 main.rs 中通过 `tauri::State<Mutex<dyn Calculator>>` 注册，
/// 前端通过 `invoke('calculate', ...)` 调用。
///
/// # 扩展方法
/// 未来添加科学计算器:
/// 1. 创建 `calc/scientific.rs`
/// 2. 实现 `impl Calculator for ScientificCalculator`
/// 3. 在 main.rs 中将 State 切换为 ScientificCalculator
pub trait Calculator: Send + Sync {
    /// 处理一个按钮动作，返回新的显示状态
    fn handle_action(&mut self, req: CalcRequest) -> CalcResponse;

    /// 重置计算器状态
    fn reset(&mut self);
}
