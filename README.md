# 简单计算器 (calc-tauri)

基于 Tauri v2 + Vue 3 + Rust 的简单计算器，用于学习 Rust 和 Tauri 框架。

## 功能

- 四则运算（加、减、乘、除）
- 百分比、正负号切换
- 链式计算（连续按操作符）
- 错误处理（除零提示）
- 深色主题，仿 iOS 计算器风格

## 架构

- **Rust 后端**：通过 `tauri::State<Mutex<BasicCalculator>>` 管理所有计算状态
- **Vue 前端**：只做 UI 渲染，通过 `invoke()` 调用 Rust 命令
- **Calculator trait**：抽象接口，预留科学计算扩展

```
用户点击按钮 → invoke('calculate') → Rust State 处理 → 返回结果 → 更新显示
```

## 快速开始

```bash
# 安装依赖
npm install

# 开发模式
npm run tauri dev

# 生产构建
npm run tauri build
```

## 技术栈

| 技术 | 版本 | 用途 |
|------|------|------|
| Tauri | v2 | 桌面应用框架 |
| Rust | 2021 Edition | 后端计算逻辑 |
| Vue 3 | 3.4 | 前端 UI |
| TypeScript | 5.4 | 前端类型安全 |
| Vite | 5.2 | 前端构建工具 |

## 项目结构

```
src/components/Calculator.vue     # 计算器 UI 组件
src-tauri/src/lib.rs              # Tauri 入口，注册命令和状态
src-tauri/src/calc/mod.rs         # Calculator trait 定义
src-tauri/src/calc/basic.rs       # 基础四则运算实现
src-tauri/src/calc/error.rs       # 错误类型
```

## 文档

详细学习文档位于 `~/workspace/项目管理/开发文档/简单计算器/rust版/`：

```
00_项目概述/
  01_项目介绍.org           # 架构、数据流、目录结构
  02_技术栈说明.org         # 技术选型、学习要点总结
  03_开发部署手册.org       # 环境配置、启动流程、常见问题
04_开发实现/
  01_Rust后端-技术方案.org  # Rust 模块设计、状态机、错误处理
  02_前端UI-技术方案.org    # Vue 组件、样式、invoke 调用
  03_Tauri通信机制-技术方案.org  # IPC 原理、State、serde、权限系统
```
