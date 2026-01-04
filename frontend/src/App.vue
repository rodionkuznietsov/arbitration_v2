<template>
  <div id="app">
    <div id="header">
      <div id="status">
        <div id="status_cirle"></div>
        <span>Offline</span>
      </div>
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
  margin-left: 20px;
  margin-top: 20px;
}

#status {
  display: flex;
  gap: 5px;
  align-items: center;
}

#status_cirle {
  background-color: red;
  width: 16px;
  height: 16px;
  border-radius: 50%;
}

#app {
  font-family: Avenir, Helvetica, Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  text-align: left;
  color: #ffffff;
  font-size: 16px;
}

body {
  background-color: #121212;
}
</style>
