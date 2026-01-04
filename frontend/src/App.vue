<template>
  <div id="app">
    <AppHeader />

    <!-- <div id="order_price">
      <span>Вход</span>
      <span>Выход</span>
    </div> -->

    <form id="form">
      <div class="form-group">
        <label for="ticker">Тикер (BTC):</label>
        <input id="ticker" name="ticker" type="text" value="BTC" class="form_input">
      </div>

      <div class="form-group">
        <label for="order">Ордер (USDT):</label>
        <input id="order" name="order" type="text" value="0.00" class="form_input">
      </div>
    </form>

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
#form {
  margin-top: 20px;
  display: grid;
  gap: 20px;
  top: 0;
  left: 20px;
  right: 20px;
  justify-content: space-around;
  grid-template-columns: repeat(auto-fit, minmax(100px, 1fr));
}

.form_input {
  padding: 18px;
  border: 1px solid #54555a;
  color: #ffffff;
  font-weight: 600;
  border-radius: 8px;
  font-size: 15px;
  margin-top: 10px;
  background-color: #23262b;
}

#footer {
  display: flex;
  gap: 20px;
  align-items: center;
  margin-top: 5px;
  position: absolute;
  justify-content: space-around;
  bottom: 0;
  left: 20px;
  right: 20px;
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
