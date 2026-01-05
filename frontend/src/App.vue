<template>
  <div id="app">
    <AppHeader />

    <!-- <div id="order_price">
      <span>Вход</span>
      <span>Выход</span>
    </div> -->

    <form id="form">
      <div id="form-column">
        <div class="form-group">
          <label for="ticker" id="form_label">Тикер (BTC):</label>
          <input id="ticker" name="ticker" type="text" value="BTC" class="form_input">
        </div>

        <div class="form-group">
          <label for="order" id="form_label">Лонг:</label>
          <input id="order" name="order" type="number" value="0.00" class="form_input">
        </div>

        <div class="form-group">
          <input id="combobox" v-model="type" type="text" @focus="show = true" @input="show = true" readonly="true" placeholder="Bybit">
          <ul class="combobox-list" id="optionsList" v-show="show">
            <li id="combobox_element" @mousedown="select('Bybit')">Bybit</li>
            <li id="combobox_element" @mousedown="select('Mexc')">Mexc</li>
          </ul>
        </div>
      </div>

      <div id="form-column">
        <div class="form-group">
          <label for="ticker" id="form_label">Ордер (USDT):</label>
          <input id="ticker" name="ticker" type="text" value="0.00" class="form_input">
        </div>

        <div class="form-group">
          <label for="order" id="form_label">Шорт:</label>
          <input id="order" name="order" type="number" value="0.00" class="form_input">
        </div>

        <div class="form-group">
          <input id="combobox" v-model="type" type="text" @focus="show = true" @input="show = true" readonly="true" placeholder="Mexc">
          <ul class="combobox-list" id="optionsList" v-show="show">
            <li id="combobox_element" @mousedown="select('Bybit')">Bybit</li>
            <li id="combobox_element" @mousedown="select('Mexc')">Mexc</li>
          </ul>
        </div>
      </div>
    </form>

    <footer id="footer">
      <button id="start">Старт</button>
      <button id="stop">Стоп</button>
    </footer>
  </div>
</template>

<script setup>
import { onMounted, ref } from 'vue'
import WebApp from "@twa-dev/sdk"
import AppHeader from './components/AppHeader.vue'

const type = ref('')
const show = ref(false)

onMounted(() => {
  WebApp.ready()
  WebApp.expand()
  console.log('start_param:', WebApp.initDataUnsafe.start_param)
})

const select = (v) => {
  type.value = v
  show.value = false
}

</script>

<style>
#combobox {
  width: 100%;
  padding: 15px;
  border: 1px solid #54555a;
  color: #ffffff;
  font-weight: 600;
  border-radius: 8px;
  font-size: 20px;
  margin-top: 10px;
  background-color: #23262b;
  outline: 1px solid #65666d;
  box-sizing: border-box;
  cursor: pointer;
  text-transform: uppercase;
}

.combobox-list {
  background-color: #23262b;
  border-radius: 8px;
  padding: 15px;
}

#combobox_element {
  cursor: pointer;
  padding: 5px;
  border-radius: 8px;
  list-style: none;
  margin: 0;
  padding-left: 0;
  text-align: left;
}

#combobox_element:hover {
  filter: opacity(75%);
  background-color: #54555a;
}

#form {
  margin-top: 20px;
  display: grid;
  gap: 20px;
  padding: 0 10px;
  width: 100%;
  box-sizing: border-box;
  justify-content: space-around;
  grid-template-columns: repeat(auto-fit, minmax(100px, 1fr));
}

#form-column{
  display: flex;
  gap: 20px;
  flex-direction: column;
}

.form_input {
  width: 100%;
  padding: 15px;
  border: 1px solid #54555a;
  color: #ffffff;
  font-weight: 600;
  border-radius: 8px;
  font-size: 20px;
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
  justify-content: flex-start;
  font-size: 20px;
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
  background-color: rgb(151, 15, 15);
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
