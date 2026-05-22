// 基础计算器实现
// 支持四则运算（加减乘除）、百分比、正负号切换、小数点
//
// 核心设计：
// - BasicCalculator 持有计算器的全部状态（当前值、上一个操作数、操作符）
// - 通过 tauri::State<Mutex<BasicCalculator>> 在 Tauri 中注册为全局状态
// - 前端每次按钮点击只发送动作，由 Rust 维护和更新状态
//
// 状态机模型：
//   初始状态: current="0", previous=None, operator=None
//   输入数字: current 追加数字
//   按操作符: current → previous, 记录操作符
//   按等号:   previous op current → current, 清除 previous 和 operator
//   按清除:   回到初始状态

use super::error::CalcError;
use super::{CalcRequest, CalcResponse, Calculator};

/// 基础计算器
///
/// 维护计算器运行时状态，实现 Calculator trait。
/// 通过 Mutex 包裹后在 Tauri 中作为全局状态使用，
/// 保证多线程安全访问。
pub struct BasicCalculator {
    /// 当前正在输入的数字字符串（如 "12"、"3.14"）
    current: String,

    /// 上一个操作数：按下操作符时，current 的值会被解析为 f64 存到这里
    previous: Option<f64>,

    /// 当前操作符: "+", "-", "×", "÷"
    operator: Option<String>,

    /// 表达式行显示内容（如 "12 + 3 ="）
    expression: String,

    /// 标记是否刚完成了一次运算（按了等号）
    /// 为 true 时，下次输入数字会清除当前值开始新输入
    just_evaluated: bool,
}

impl BasicCalculator {
    /// 创建新的计算器实例，初始状态为 "0"
    pub fn new() -> Self {
        Self {
            current: String::from("0"),
            previous: None,
            operator: None,
            expression: String::new(),
            just_evaluated: false,
        }
    }

    /// 处理数字输入 (0-9)
    ///
    /// - 如果刚完成运算或当前显示 "0"，开始新的输入
    /// - 否则追加到当前数字末尾
    /// - 限制输入长度为 16 位，防止溢出
    fn handle_number(&mut self, digit: &str) -> CalcResponse {
        if self.just_evaluated || self.current == "0" {
            self.current = digit.to_string();
            self.just_evaluated = false;
        } else {
            // 限制输入长度
            if self.current.len() >= 16 {
                return self.make_response();
            }
            self.current.push_str(digit);
        }
        self.make_response()
    }

    /// 处理小数点输入
    ///
    /// - 如果刚完成运算，开始新的输入 "0."
    /// - 已有小数点则忽略（防止 "3..1" 这种输入）
    fn handle_decimal(&mut self) -> CalcResponse {
        if self.just_evaluated {
            self.current = String::from("0.");
            self.just_evaluated = false;
            return self.make_response();
        }
        if !self.current.contains('.') {
            self.current.push('.');
        }
        self.make_response()
    }

    /// 处理操作符 (+, -, ×, ÷)
    ///
    /// - 如果有待计算的表达式且不是刚完成运算，先执行上一次运算（链式计算）
    /// - 将当前值存为 previous，记录操作符，重置 current 为 "0"
    /// - 更新表达式行显示
    fn handle_operator(&mut self, op: &str) -> CalcResponse {
        // 链式计算：如果已经有 previous 和 operator，先算出中间结果
        if self.previous.is_some() && self.operator.is_some() && !self.just_evaluated {
            let result = self.evaluate();
            match result {
                Ok(val) => {
                    self.current = format_number(val);
                    self.previous = Some(val);
                }
                Err(e) => {
                    let resp = CalcResponse {
                        display: String::from("错误"),
                        expression: e.to_string(),
                        error: Some(e.to_string()),
                    };
                    self.reset();
                    return resp;
                }
            }
        } else {
            self.previous = Some(self.current.parse::<f64>().unwrap_or(0.0));
        }

        self.operator = Some(op.to_string());
        self.expression = format!("{} {}", format_number(self.previous.unwrap()), op);
        self.just_evaluated = false;
        self.current = String::from("0");
        self.make_response()
    }

    /// 处理等号
    ///
    /// - 取出 previous、operator 和 current，执行二元运算
    /// - 将结果显示在显示屏上，表达式行显示完整算式
    /// - 重置 previous 和 operator，标记 just_evaluated
    fn handle_equals(&mut self) -> CalcResponse {
        // 没有操作符时忽略
        if self.operator.is_none() || self.previous.is_none() {
            return self.make_response();
        }

        let prev = self.previous.unwrap();
        let curr = self.current.parse::<f64>().unwrap_or(0.0);
        let op = self.operator.clone().unwrap();

        match self.compute(prev, curr, &op) {
            Ok(result) => {
                self.expression =
                    format!("{} {} {} =", format_number(prev), op, format_number(curr));
                let display = format_number(result);
                self.current = display.clone();
                self.previous = None;
                self.operator = None;
                self.just_evaluated = true;
                CalcResponse {
                    display,
                    expression: self.expression.clone(),
                    error: None,
                }
            }
            Err(e) => {
                let resp = CalcResponse {
                    display: String::from("错误"),
                    expression: self.expression.clone(),
                    error: Some(e.to_string()),
                };
                self.reset();
                resp
            }
        }
    }

    /// 处理清除（C 键）— 重置所有状态
    fn handle_clear(&mut self) -> CalcResponse {
        self.reset();
        self.make_response()
    }

    /// 处理百分比 — 将当前值除以 100
    fn handle_percent(&mut self) -> CalcResponse {
        let val = self.current.parse::<f64>().unwrap_or(0.0);
        self.current = format_number(val / 100.0);
        self.make_response()
    }

    /// 处理正负号切换
    ///
    /// - 如果当前值为 "0"，不做任何操作
    /// - 有负号则去掉，无负号则添加
    fn handle_negate(&mut self) -> CalcResponse {
        if self.current != "0" {
            if self.current.starts_with('-') {
                self.current.remove(0);
            } else {
                self.current.insert(0, '-');
            }
        }
        self.make_response()
    }

    /// 执行二元运算的核心函数
    ///
    /// 使用 Rust 的 match 模式匹配来分发不同操作符，
    /// 这是 Rust 中替代 if-else 链的惯用写法。
    fn compute(&self, a: f64, b: f64, op: &str) -> Result<f64, CalcError> {
        match op {
            "+" => Ok(a + b),
            "-" => Ok(a - b),
            "×" => Ok(a * b),
            "÷" => {
                // Rust 的 Result 要求显式处理可能的错误
                if b == 0.0 {
                    Err(CalcError::DivisionByZero)
                } else {
                    Ok(a / b)
                }
            }
            _ => Err(CalcError::InvalidInput(format!("未知操作符: {}", op))),
        }
    }

    /// 执行上一次待计算的运算（链式计算时使用）
    fn evaluate(&mut self) -> Result<f64, CalcError> {
        let prev = self.previous.unwrap();
        let curr = self.current.parse::<f64>().unwrap_or(0.0);
        let op = self.operator.clone().unwrap();
        self.compute(prev, curr, &op)
    }

    /// 从当前状态构造 CalcResponse
    fn make_response(&self) -> CalcResponse {
        CalcResponse {
            display: self.current.clone(),
            expression: self.expression.clone(),
            error: None,
        }
    }
}

/// 实现 Calculator trait
///
/// 这是 Rust 的 trait 实现语法，类似于其他语言的"接口实现"。
/// handle_action 使用 match 将不同 action 分发到对应处理函数。
impl Calculator for BasicCalculator {
    fn handle_action(&mut self, req: CalcRequest) -> CalcResponse {
        match req.action.as_str() {
            "number" => self.handle_number(&req.value),
            "operator" => self.handle_operator(&req.value),
            "equals" => self.handle_equals(),
            "clear" => self.handle_clear(),
            "percent" => self.handle_percent(),
            "negate" => self.handle_negate(),
            "decimal" => self.handle_decimal(),
            _ => CalcResponse {
                display: self.current.clone(),
                expression: self.expression.clone(),
                error: Some(format!("未知动作: {}", req.action)),
            },
        }
    }

    fn reset(&mut self) {
        self.current = String::from("0");
        self.previous = None;
        self.operator = None;
        self.expression = String::new();
        self.just_evaluated = false;
    }
}

/// 格式化数字显示
///
/// - 整数：去掉小数部分（3.0 → "3"）
/// - 浮点数：保留最多 10 位小数，去掉尾部多余的零（3.1400 → "3.14"）
/// - 特殊处理：避免显示 "-0"
fn format_number(num: f64) -> String {
    // 整数且在安全范围内，直接转为整数格式
    if num.fract() == 0.0 && num.abs() < 1e15 {
        format!("{}", num as i64)
    } else {
        // 浮点数：保留 10 位小数，去掉尾零
        let formatted = format!("{:.10}", num);
        let trimmed = formatted.trim_end_matches('0').trim_end_matches('.');
        trimmed.to_string()
    }
}
