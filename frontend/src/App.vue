<template>
  <div id="app">
    <h1>Arbitration V1</h1>
    <p>Start param: {{ startParam }}</p>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import WebApp from "@twa-dev/sdk"

const startParam = ref('')

onMounted(() => {
  WebApp.ready()
  WebApp.expand()
  startParam.value = WebApp.initDataUnsafe.start_param || 'none'
  console.log('start_param:', WebApp.initDataUnsafe.start_param)
})
</script>

<style>
#app {
  font-family: Avenir, Helvetica, Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  text-align: center;
  color: #2c3e50;
  margin-top: 60px;
}
</style>
