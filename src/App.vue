<script setup lang="ts">
import Calculator from './components/Calculator.vue'
</script>

<template>
  <Calculator />
</template>

<style>
/* 全局样式：让计算器撑满窗口 */
html, body, #app {
  margin: 0;
  padding: 0;
  height: 100%;
  overflow: hidden;
}
</style>
