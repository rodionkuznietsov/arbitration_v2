<template>
  <div id="app">
    <AppHeader />

    <div>Вход</div>
    <div>Выход</div>

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
  bottom: 0px;
}

#start {
  background-color: green;
  padding: 8px;
  border: none;

}

#stop {
  background-color: red;
  padding: 8px;
  border: none;

}

#app {
  font-family: Avenir, Helvetica, Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  text-align: left;
  color: #ffffff;
  font-size: 16px;
  margin-left: 5px;
  margin-top: 5px;
  margin-bottom: 5px;
  margin-right: 5px;
}

body {
  background-color: #121212;
}
</style>
