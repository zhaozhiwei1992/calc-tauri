// 自定义错误类型
// 用于计算过程中可能出现的异常情况
//
// Rust 的 Result<T, E> 模式强制开发者处理可能的错误，
// 这是 Rust 保证安全性的重要机制之一。

/// 计算器错误枚举
///
/// 使用 enum 定义所有可能的错误类型，
/// 配合 Result<T, CalcError> 在运算函数间传播错误。
#[derive(Debug)]
pub enum CalcError {
    /// 除数为零 — 数学上未定义的操作
    DivisionByZero,

    /// 数值溢出 — 计算结果超出 f64 表示范围
    Overflow,

    /// 无效输入 — 如多个小数点、未知操作符等
    InvalidInput(String),
}

/// 实现 Display trait，提供用户友好的中文错误描述
///
/// Display trait 类似其他语言的 toString()，
/// 用于将错误转换为可读字符串显示在 UI 上。
impl std::fmt::Display for CalcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CalcError::DivisionByZero => write!(f, "除数不能为零"),
            CalcError::Overflow => write!(f, "数值溢出"),
            CalcError::InvalidInput(msg) => write!(f, "无效输入: {}", msg),
        }
    }
}
