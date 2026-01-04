<template>
  <div id="app">
    <div id="header">
      <div id="online">Offline</div>
    </div>
  </div>
</template>

<script setup>
import { onMounted } from 'vue'
import WebApp from "@twa-dev/sdk"

onMounted(() => {
  WebApp.ready()
  WebApp.expand()
  console.log('start_param:', WebApp.initDataUnsafe.start_param)
})
</script>

<style>
#header {
  display: flex;
  margin-left: 10px;
  margin-top: 10px;
}

#app {
  font-family: Avenir, Helvetica, Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  text-align: center;
  color: #2c3e50;
}
</style>
