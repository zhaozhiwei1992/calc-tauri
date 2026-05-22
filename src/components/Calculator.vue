<!-- 计算器主组件 -->
<!-- 职责：UI 渲染 + 按钮事件收集 -->
<!-- 所有计算逻辑通过 Tauri IPC 交给 Rust 后端处理 -->

<script setup lang="ts">
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

// 显示屏状态 — 仅用于渲染，不做计算
const display = ref('0')
const expression = ref('')

// 按钮点击处理
// 将按钮动作发送到 Rust 后端，由 tauri::State 维护的计算器处理
async function handleButton(action: string, value: string = '') {
  try {
    const response = await invoke<{
      display: string
      expression: string
      error: string | null
    }>('calculate', { req: { action, value } })
    display.value = response.display
    expression.value = response.expression
  } catch (e) {
    console.error('调用 Rust 计算命令失败:', e)
  }
}

// 按钮布局定义：从上到下、从左到右排列
// class 用于区分数字键(num)、功能键(fn)、操作符键(op)
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
    <!-- 按钮网格 -->
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
/* 计算器整体容器 - 深色主题，仿 iOS 计算器风格 */
.calculator {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: #1c1c1c;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  user-select: none;
}

/* 显示屏：右对齐，底部对齐 */
.display {
  flex: 1;
  display: flex;
  flex-direction: column;
  justify-content: flex-end;
  align-items: flex-end;
  padding: 12px 16px;
  min-height: 80px;
}

/* 表达式行：显示历史运算 */
.expression {
  font-size: 14px;
  color: #888;
  min-height: 20px;
}

/* 主数字显示 */
.display-value {
  font-size: 40px;
  color: #fff;
  font-weight: 300;
  word-break: break-all;
  text-align: right;
}

/* 按钮网格：4 列等宽 */
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

/* 数字键：深灰 */
.num {
  background: #333;
  color: #fff;
}

/* 功能键 (C, ±, %)：浅灰 */
.fn {
  background: #a5a5a5;
  color: #1c1c1c;
  font-size: 18px;
}

/* 操作符键 (÷, ×, -, +, =)：橙色 */
.op {
  background: #f09a36;
  color: #fff;
  font-size: 24px;
}

/* 0 键横跨两列 */
.wide {
  grid-column: span 2;
  text-align: left;
  padding-left: 24px;
}
</style>
