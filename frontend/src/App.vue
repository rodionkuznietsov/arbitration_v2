<template>
  <div id="app">
    <AppHeader />

    <!-- <div id="order_price">
      <span>Вход</span>
      <span>Выход</span>
    </div> -->

    <form id="form">
      <div class="form-group">
        <label for="ticker" id="form_label">Тикер (BTC):</label>
        <input id="ticker" name="ticker" type="text" value="BTC" class="form_input">
      </div>

      <div class="form-group">
        <label for="order" id="form_label">Ордер (USDT):</label>
        <input id="order" name="order" type="number" value="0.00" class="form_input">
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
  padding: 0 20px;
  width: 100%;
  box-sizing: border-box;
  justify-content: space-around;
  grid-template-columns: repeat(auto-fit, minmax(100px, 1fr));
}

.form_input {
  width: 100%;
  padding: 15px;
  border: 1px solid #54555a;
  color: #ffffff;
  font-weight: 600;
  border-radius: 8px;
  font-size: 14px;
  margin-top: 10px;
  background-color: #23262b;
  outline: 1px solid #65666d;
  box-sizing: border-box;
}

.form-group {
  display: flex;
  flex-direction: column;
}

#form_label {
  display: flex;
  justify-content: center;
  font-size: 14px;
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
