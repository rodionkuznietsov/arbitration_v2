<template>
  <div id="app">
    <AppHeader />

    <div>Вход</div>
    <div>Выход</div>


    <div>
      <button>Старт</button>
      <button>Стоп</button>
    </div>
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
