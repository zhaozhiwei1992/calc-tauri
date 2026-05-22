# 简单计算器实现计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 基于 Tauri v2 + Vue 3 构建一个简单计算器，Rust 负责所有计算逻辑（通过 `tauri::State` 管理），前端只做 UI 展示。

**Architecture:** 前端 Vue 3 组件通过 Tauri IPC (`invoke`) 调用 Rust 命令，每次只传递按钮动作，Rust 通过 `tauri::State` 维护计算器内部状态，返回显示内容。Rust 侧用 trait 抽象计算器接口，预留科学计算扩展。

**Tech Stack:** Tauri v2, Vue 3.4 + TypeScript, Vite 5, Rust 2021 Edition

---

### Task 1: 项目配置调整

**Files:**
- Modify: `src-tauri/tauri.conf.json`
- Modify: `vite.config.ts`

**Step 1: 修复 Vite 端口不匹配**

`tauri.conf.json` 中 `devPath` 指向 `localhost:8080`，但 Vite 默认端口是 5173。修改 `vite.config.ts` 将端口设为 8080：

```ts
// vite.config.ts
export default defineConfig({
  plugins: [vue(), VueDevTools()],
  server: {
    port: 8080,
  },
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url))
    }
  }
})
```

**Step 2: 更新 Tauri 窗口配置**

修改 `src-tauri/tauri.conf.json` 中 windows 配置为计算器窗口大小：

```json
"windows": [
  {
    "fullscreen": false,
    "height": 480,
    "resizable": false,
    "title": "计算器",
    "width": 320
  }
]
```

同时将 `identifier` 改为 `"com.calc.tauri"`。

**Step 3: Commit**

```bash
git add vite.config.ts src-tauri/tauri.conf.json
git commit -m "chore: 调整项目配置适配计算器窗口"
```

---

### Task 2: 清理模板代码

**Files:**
- Modify: `src/main.ts` — 移除 Pinia 和 Router
- Modify: `src/App.vue` — 简化为只渲染 Calculator
- Delete: `src/components/HelloWorld.vue`, `src/components/TheWelcome.vue`, `src/components/WelcomeItem.vue`, `src/components/icons/` 整个目录
- Delete: `src/router/` 目录
- Delete: `src/stores/` 目录
- Delete: `src/views/` 目录
- Modify: `index.html` — title 改为 "计算器"

**Step 1: 简化 main.ts**

```ts
import './assets/main.css'
import { createApp } from 'vue'
import App from './App.vue'

createApp(App).mount('#app')
```

**Step 2: 简化 App.vue**

```vue
<script setup lang="ts">
import Calculator from './components/Calculator.vue'
</script>

<template>
  <Calculator />
</template>

<style>
/* 全局样式重置：让计算器撑满窗口 */
html, body, #app {
  margin: 0;
  padding: 0;
  height: 100%;
  overflow: hidden;
}
</style>
```

**Step 3: 删除不需要的模板文件**

```bash
rm -rf src/components/HelloWorld.vue src/components/TheWelcome.vue src/components/WelcomeItem.vue src/components/icons src/router src/stores src/views
```

**Step 4: 更新 index.html title**

将 `<title>Vite App</title>` 改为 `<title>计算器</title>`。

**Step 5: Commit**

```bash
git add -A
git commit -m "chore: 清理 Vue 模板代码，精简为计算器项目"
```

---

### Task 3: Rust 后端 - 错误类型和数据结构

**Files:**
- Create: `src-tauri/src/calc/mod.rs`
- Create: `src-tauri/src/calc/error.rs`

**Step 1: 创建 calc 模块和错误类型**

`src-tauri/src/calc/mod.rs`：

```rust
// 计算器模块
// 提供计算器 trait 定义和基础/科学计算器实现

pub mod error;
pub mod basic;

use serde::{Deserialize, Serialize};

/// 前端发来的按钮动作请求
/// 每次用户点击按钮，前端只发送一个 action + value
#[derive(Debug, Deserialize)]
pub struct CalcRequest {
    /// 动作类型: "number" | "operator" | "equals" | "clear" | "percent" | "negate" | "decimal"
    pub action: String,
    /// 具体值: 数字键传 "0"-"9"，操作符传 "+"/"-"/"×"/"÷"，其他动作可为空
    pub value: String,
}

/// 返回给前端的计算结果
#[derive(Debug, Serialize)]
pub struct CalcResponse {
    /// 显示屏主数字
    pub display: String,
    /// 表达式行（显示历史运算）
    pub expression: String,
    /// 错误信息（除零等情况）
    pub error: Option<String>,
}

/// 计算器 trait - 所有计算器类型的统一接口
/// BasicCalculator 和未来的 ScientificCalculator 都实现此 trait
pub trait Calculator: Send + Sync {
    /// 处理一个按钮动作，返回新的显示状态
    fn handle_action(&mut self, req: CalcRequest) -> CalcResponse;
    /// 重置计算器状态
    fn reset(&mut self);
}
```

`src-tauri/src/calc/error.rs`：

```rust
// 自定义错误类型
// 用于计算过程中可能出现的错误

/// 计算器错误枚举
#[derive(Debug)]
pub enum CalcError {
    /// 除数为零
    DivisionByZero,
    /// 数值溢出
    Overflow,
    /// 无效输入（如多个小数点）
    InvalidInput(String),
}

impl std::fmt::Display for CalcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CalcError::DivisionByZero => write!(f, "除数不能为零"),
            CalcError::Overflow => write!(f, "数值溢出"),
            CalcError::InvalidInput(msg) => write!(f, "无效输入: {}", msg),
        }
    }
}
```

**Step 2: Commit**

```bash
git add src-tauri/src/calc/
git commit -m "feat(rust): 添加计算器模块、数据结构和错误类型"
```

---

### Task 4: Rust 后端 - 基础计算器实现

**Files:**
- Create: `src-tauri/src/calc/basic.rs`

**Step 1: 实现 BasicCalculator**

```rust
// 基础计算器实现
// 支持四则运算（加减乘除）、百分比、正负号切换

use super::error::CalcError;
use super::{CalcRequest, CalcResponse, Calculator};

/// 基础计算器状态
/// 维护当前输入、上一个操作数和待执行的操作符
pub struct BasicCalculator {
    /// 当前正在输入的数字字符串
    current: String,
    /// 上一个操作数（按操作符时保存的值）
    previous: Option<f64>,
    /// 当前操作符: "+", "-", "×", "÷"
    operator: Option<String>,
    /// 表达式行显示内容
    expression: String,
    /// 是否刚完成了一次运算（按了等号），下次输入数字时清除当前值
    just_evaluated: bool,
}

impl BasicCalculator {
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
    fn handle_number(&mut self, digit: &str) -> CalcResponse {
        // 如果刚完成运算或当前显示为 "0"，开始新的输入
        if self.just_evaluated || self.current == "0" {
            self.current = digit.to_string();
            self.just_evaluated = false;
        } else {
            // 限制输入长度为 16 位，防止溢出
            if self.current.len() >= 16 {
                return self.make_response();
            }
            self.current.push_str(digit);
        }
        self.make_response()
    }

    /// 处理小数点输入
    fn handle_decimal(&mut self) -> CalcResponse {
        if self.just_evaluated {
            // 运算完成后按小数点，开始新的输入 "0."
            self.current = String::from("0.");
            self.just_evaluated = false;
            return self.make_response();
        }
        // 已有小数点则忽略
        if !self.current.contains('.') {
            self.current.push('.');
        }
        self.make_response()
    }

    /// 处理操作符 (+, -, ×, ÷)
    fn handle_operator(&mut self, op: &str) -> CalcResponse {
        // 如果有待计算的表达式，先执行上一次运算
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
                self.expression = format!("{} {} {} =", format_number(prev), op, format_number(curr));
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

    /// 处理清除
    fn handle_clear(&mut self) -> CalcResponse {
        self.reset();
        self.make_response()
    }

    /// 处理百分比
    fn handle_percent(&mut self) -> CalcResponse {
        let val = self.current.parse::<f64>().unwrap_or(0.0);
        self.current = format_number(val / 100.0);
        self.make_response()
    }

    /// 处理正负号切换
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

    /// 执行二元运算
    fn compute(&self, a: f64, b: f64, op: &str) -> Result<f64, CalcError> {
        match op {
            "+" => Ok(a + b),
            "-" => Ok(a - b),
            "×" => Ok(a * b),
            "÷" => {
                if b == 0.0 {
                    Err(CalcError::DivisionByZero)
                } else {
                    Ok(a / b)
                }
            }
            _ => Err(CalcError::InvalidInput(format!("未知操作符: {}", op))),
        }
    }

    /// 执行上一次待计算的运算（连续按操作符时使用）
    fn evaluate(&mut self) -> Result<f64, CalcError> {
        let prev = self.previous.unwrap();
        let curr = self.current.parse::<f64>().unwrap_or(0.0);
        let op = self.operator.clone().unwrap();
        self.compute(prev, curr, &op)
    }

    /// 构造响应
    fn make_response(&self) -> CalcResponse {
        CalcResponse {
            display: self.current.clone(),
            expression: self.expression.clone(),
            error: None,
        }
    }
}

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

/// 格式化数字显示：去掉不必要的尾零，限制小数位数
fn format_number(num: f64) -> String {
    // 检查是否为整数
    if num.fract() == 0.0 && num.abs() < 1e15 {
        format!("{}", num as i64)
    } else {
        // 保留最多 10 位小数，去掉尾零
        let formatted = format!("{:.10}", num);
        let trimmed = formatted.trim_end_matches('0').trim_end_matches('.');
        trimmed.to_string()
    }
}
```

**Step 2: Commit**

```bash
git add src-tauri/src/calc/basic.rs
git commit -m "feat(rust): 实现基础四则运算计算器逻辑"
```

---

### Task 5: Rust 后端 - 注册 Tauri 命令和 State

**Files:**
- Modify: `src-tauri/src/main.rs`

**Step 1: 更新 main.rs 注册命令和状态**

```rust
// 防止 Windows 发布版出现额外的控制台窗口，勿删！
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod calc;

use calc::basic::BasicCalculator;
use calc::{CalcRequest, CalcResponse, Calculator};
use std::sync::Mutex;

/// Tauri 命令：处理计算器按钮动作
/// 通过 tauri::State 获取全局计算器实例，前端每次调用只传递按钮动作
#[tauri::command]
fn calculate(state: tauri::State<Mutex<BasicCalculator>>, req: CalcRequest) -> CalcResponse {
    let mut calculator = state.lock().unwrap();
    calculator.handle_action(req)
}

/// Tauri 命令：重置计算器
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

fn main() {
    tauri::Builder::default()
        // 注册全局状态：BasicCalculator 实例被 Mutex 包裹以保证线程安全
        .manage(Mutex::new(BasicCalculator::new()))
        // 注册 Tauri 命令，供前端通过 invoke 调用
        .invoke_handler(tauri::generate_handler![calculate, reset_calculator])
        .run(tauri::generate_context!())
        .expect("启动 Tauri 应用失败");
}
```

**Step 2: Commit**

```bash
git add src-tauri/src/main.rs
git commit -m "feat(rust): 注册 Tauri 命令和 State，连接前端 IPC"
```

---

### Task 6: 前端 - Calculator 组件

**Files:**
- Create: `src/components/Calculator.vue`

**Step 1: 创建计算器主组件**

```vue
<script setup lang="ts">
// 计算器主组件
// 职责：UI 渲染 + 按钮事件收集，所有计算逻辑通过 Tauri IPC 交给 Rust 处理

import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'

// 显示屏状态
const display = ref('0')         // 当前数字
const expression = ref('')       // 表达式行

// 按钮点击处理：将按钮动作发送到 Rust 后端
async function handleButton(action: string, value: string = '') {
  try {
    const response = await invoke<{ display: string; expression: string; error: string | null }>(
      'calculate',
      { req: { action, value } }
    )
    display.value = response.display
    expression.value = response.expression
  } catch (e) {
    console.error('调用 Rust 计算命令失败:', e)
  }
}

// 按钮布局定义：从上到下、从左到右
const buttons = [
  { label: 'C', action: 'clear', class: 'fn' },
  { label: '±', action: 'negate', class: 'fn' },
  { label: '%', action: 'percent', class: 'fn' },
  { label: '÷', action: 'operator', value: '÷', class: 'op' },
  { label: '7', action: 'number', value: '7', class: 'num' },
  { label: '8', action: 'number', value: '8', class: 'num' },
  { label: '9', action: 'number', value: '9', class: 'num' },
  { label: '×', action: 'operator', value: '×', class: 'op' },
  { label: '4', action: 'number', value: '4', class: 'num' },
  { label: '5', action: 'number', value: '5', class: 'num' },
  { label: '6', action: 'number', value: '6', class: 'num' },
  { label: '-', action: 'operator', value: '-', class: 'op' },
  { label: '1', action: 'number', value: '1', class: 'num' },
  { label: '2', action: 'number', value: '2', class: 'num' },
  { label: '3', action: 'number', value: '3', class: 'num' },
  { label: '+', action: 'operator', value: '+', class: 'op' },
  { label: '0', action: 'number', value: '0', class: 'num wide' },
  { label: '.', action: 'decimal', class: 'num' },
  { label: '=', action: 'equals', class: 'op' },
]
</script>

<template>
  <div class="calculator">
    <!-- 显示屏区域 -->
    <div class="display">
      <div class="expression">{{ expression }}</div>
      <div class="display-value">{{ display }}</div>
    </div>
    <!-- 按钮区域 -->
    <div class="buttons">
      <button
        v-for="btn in buttons"
        :key="btn.label"
        :class="btn.class"
        @click="handleButton(btn.action, btn.value ?? '')"
      >
        {{ btn.label }}
      </button>
    </div>
  </div>
</template>

<style scoped>
.calculator {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: #1c1c1c;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  user-select: none;
}

/* 显示屏 */
.display {
  flex: 1;
  display: flex;
  flex-direction: column;
  justify-content: flex-end;
  align-items: flex-end;
  padding: 12px 16px;
  min-height: 80px;
}

.expression {
  font-size: 14px;
  color: #888;
  min-height: 20px;
}

.display-value {
  font-size: 40px;
  color: #fff;
  font-weight: 300;
  word-break: break-all;
  text-align: right;
}

/* 按钮网格 */
.buttons {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 1px;
  padding: 1px;
}

button {
  height: 52px;
  border: none;
  border-radius: 0;
  font-size: 20px;
  cursor: pointer;
  transition: filter 0.1s;
}

button:active {
  filter: brightness(1.3);
}

/* 数字键 */
.num {
  background: #333;
  color: #fff;
}

/* 功能键 (C, ±, %) */
.fn {
  background: #a5a5a5;
  color: #1c1c1c;
  font-size: 18px;
}

/* 操作符键 (÷, ×, -, +, =) */
.op {
  background: #f09a36;
  color: #fff;
  font-size: 24px;
}

/* 0 键占两格 */
.wide {
  grid-column: span 2;
  text-align: left;
  padding-left: 24px;
}
</style>
```

**Step 2: Commit**

```bash
git add src/components/Calculator.vue
git commit -m "feat(vue): 添加计算器 UI 组件"
```

---

### Task 7: 清理前端样式和安装依赖

**Files:**
- Modify: `src/assets/base.css`
- Modify: `src/assets/main.css`

**Step 1: 简化样式文件**

清空 `src/assets/base.css` 和 `src/assets/main.css` 中的模板样式（保留最小化的 reset 即可），避免影响计算器布局。

`src/assets/main.css`：
```css
/* 计算器全局样式 */
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

html, body {
  height: 100%;
  overflow: hidden;
}
```

删除 `src/assets/base.css`。

删除 `src/assets/logo.svg`（不再需要）。

**Step 2: 安装前端依赖**

```bash
npm install
```

**Step 3: Commit**

```bash
git add -A
git commit -m "chore: 清理模板样式，安装依赖"
```

---

### Task 8: 验证构建

**Step 1: 检查 Rust 编译**

```bash
cd src-tauri && cargo check
```

预期：编译成功，无错误。

**Step 2: 验证前端编译**

```bash
npx vue-tsc --build --force
```

预期：类型检查通过。

**Step 3: 完整构建测试**

```bash
npm run build
```

预期：前后端均构建成功。

**Step 4: Commit（如有修复）**

---

### Task 9: 编写项目文档

**Files:**
- Create: `~/workspace/项目管理/开发文档/简单计算器/rust版/001_项目架构说明.md`
- Create: `~/workspace/项目管理/开发文档/简单计算器/rust版/002_Rust后端详解.md`
- Create: `~/workspace/项目管理/开发文档/简单计算器/rust版/003_Vue前端详解.md`
- Create: `~/workspace/项目管理/开发文档/简单计算器/rust版/004_Tauri通信机制.md`
- Create: `~/workspace/项目管理/开发文档/简单计算器/rust版/005_学习要点总结.md`

文档内容涵盖：
1. 项目整体架构和目录结构说明
2. Rust 模块设计（trait、状态管理、错误处理）
3. Vue 组件设计和样式方案
4. Tauri IPC 通信原理（invoke、State、序列化）
5. Rust 语法学习要点（所有权、trait、枚举、模式匹配等）

**Step 1: 逐一编写文档（内容见实现阶段）**

**Step 2: Commit**

```bash
git add -A
git commit -m "docs: 添加项目学习文档"
```
