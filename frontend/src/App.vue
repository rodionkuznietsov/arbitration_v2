<template>
  <img alt="Vue logo" src="./assets/logo.png">
</template>

<script>
import WebApp from "@twa-dev/sdk";

export default {
  name: 'App',
  components: {
    WebApp
  }
}

WebApp.ready()
WebApp.expand()

console.log('start_param:', WebApp.initDataUnsafe.start_param)

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
