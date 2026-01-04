<template>
  <div id="app">
    <AppHeader />

    <footer id="footer">
      <button id="start">Старт</button>
      <button id="stop">Стоп</button>
    </footer>
  </div>
</template>

<script setup>
import { onMounted } from 'vue'
import WebApp from "@twa-dev/sdk"
import AppHeader from './components/AppHeader.vue'

onMounted(() => {
  WebApp.ready()
  WebApp.expand()
  console.log('start_param:', WebApp.initDataUnsafe.start_param)
})

</script>

<style>
#footer {
  display: flex;
  gap: 5px;
  align-items: center;
  margin-top: 5px;
  position: absolute;
  justify-content: space-around;
  bottom: 0;
  left: 0;
  right: 0;
  padding: 10px;
}

#start, #stop {
  flex: 1;
  padding: 15px;
  border: none;
  color: #ffffff;
  font-weight: 600;
  border-radius: 8px;
  font-size: 20px;
}

#start:hover, #stop:hover {
  filter: opacity(75%);
  cursor: pointer;
}

#start {
  background-color: green;
}

#stop {
  background-color: red;
}

#app {
  font-family: Avenir, Helvetica, Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  text-align: left;
  color: #ffffff;
  font-size: 20px;
  margin-left: 5px;
  margin-top: 5px;
  margin-bottom: 5px;
  margin-right: 5px;
}

body {
  background-color: #121212;
}
</style>
