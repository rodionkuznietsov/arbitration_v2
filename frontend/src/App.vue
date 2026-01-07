<template>
  <div id="app">
    <AppHeader ref="header"/>

    <form id="form">
      <div id="form-column">
        <div class="form-group">
          <label for="ticker" id="form_label">Тикер (BTC):</label>
          <input id="ticker" name="ticker" type="text" value="BTC" class="form_input">
        </div>

        <div class="form-group">
          <div id="form_label-with_icon">
            <label for="order" id="form_label">Лонг:</label>
            <img src="./assets/icons/up.svg" alt="" draggable="false">
          </div>
          <FormCombobox v-model="longExchange" placeholder="Bybit" :options="exchanges"/>
        </div>

        <div class="form-group">
          <FormCombobox v-model="longOrderType" placeholder="Фьючерс" :options="market_types"/>
        </div>

        <div class="form-group">
          <label for="order" id="form_label">Порог входа (%):</label>
          <input id="order" name="order" type="number" value="0.00" class="form_input">
        </div>
      </div>

      <div id="form-column">
        <div class="form-group">
          <label for="ticker" id="form_label">Ордер (USDT):</label>
          <input id="ticker" name="ticker" type="text" value="0.00" class="form_input">
        </div>

        <div class="form-group">
          <div id="form_label-with_icon">
            <label for="order" id="form_label">Шорт:</label>
            <img class="img_reverse" src="./assets/icons/up.svg" alt="" draggable="false">
          </div>
          <FormCombobox v-model="shortExchange" placeholder="Mexc" :options="exchanges"/>
        </div>

        <div class="form-group">
          <FormCombobox v-model="shortOrderType" placeholder="Фьючерс" :options="market_types"/>
        </div>

        <div class="form-group">
          <label for="order" id="form_label">Порог выхода (%):</label>
          <input id="order" name="order" type="number" value="0.00" class="form_input">
        </div>
      </div>
    </form>

    <footer id="footer">
        <div id="run_buttons">
          <button id="start" @click="start">Старт</button>
          <button id="stop" @click="stop">Стоп</button>
        </div>

      <OrderBook ref="orderBook"/>
      <AppMenu />
    </footer>
  </div>
</template>

<script setup>
import { onMounted, ref } from 'vue'
import WebApp from "@twa-dev/sdk"
import AppHeader from './components/AppHeader.vue'
import FormCombobox from './components/FormCombobox.vue';
import OrderBook from './components/OrderBook.vue';
import AppMenu from './components/AppMenu.vue';

const exchanges = ["Bybit", "Mexc"]
const market_types = ["Спот", "Фьючерс"]

onMounted(() => {
  WebApp.ready()
  WebApp.expand()
})

const orderBook = ref(null)
const header = ref(null)
const longExchange = ref("")
const longOrderType = ref("")

const shortExchange = ref("")
const shortOrderType = ref("")

function start() {
  if (header.value) {
    header.value.change_work_status(true)
  }
  console.log("Long Exchange: ", longExchange.value)
  console.log("Long OrderType: ", longOrderType.value)

  console.log("Short Exchange: ", shortExchange.value)
  console.log("Short OrderType: ", shortOrderType.value)

  orderBook.value.exchanges(longExchange.value, shortExchange.value)
  orderBook.value.show()  
  orderBook.value.start()
}

function stop() {
  orderBook.value.stop()
    if (header.value) {
      header.value.change_work_status(false)
    }
}

</script>

<style>
  @import url('https://fonts.googleapis.com/css2?family=PT+Serif:ital,wght@0,400;0,700;1,400;1,700&family=Vollkorn+SC:wght@400;600;700;900&display=swap');

  #form {
    margin-top: 20px;
    display: grid;
    gap: 10px;
    padding-left: 10px;
    padding-right: 10px;
    width: 100%;
    box-sizing: border-box;
    justify-content: space-around;
    grid-template-columns: repeat(auto-fit, minmax(100px, 1fr));
  }

  #form_label-with_icon {
    display: flex;
    flex-direction: row;
    align-items: center;
    justify-content: flex-start;
    gap: 10px;
  }

  #form_label-with_icon img {
    width: 16px;
    height: 16px;
    margin-top: 12px;
  }

  .img_reverse {
    transform: rotate(60deg);
  }

  #form-column{
    display: flex;
    gap: 5px;
    flex-direction: column;
  }

  .form_input {
    width: 100%;
    padding: 8px;
    border: 1px solid rgba(48, 60, 81, 0.6);;
    color: #ffffff;
    border-radius: 8px;
    font-size: 16px;
    margin-top: 10px;
    background-color: rgba(48, 60, 81, 0.6);
    outline: none;
    box-sizing: border-box;
  }

  .form-group {
    display: flex;
    flex-direction: column;
  }

  #form_label {
    display: flex;
    justify-content: flex-start;
    font-size: 16px;
    margin-top: 10px;
  }

  #footer {
    display: flex;
    flex-direction: column;
    margin-top: 10px;
    padding: 8px;
    overflow-y: visible;
    margin-bottom: 70px;
  }

  #run_buttons {
    display: flex;
    gap: 10px;
  }

  #start, #stop {
    flex: 1;
    padding: 8px;
    border: none;
    color: #ffffff;
    border-radius: 8px;
    font-size: 16px;
    transition: all 0.6s;
  }

  #start:hover, #stop:hover {
    filter: opacity(75%);
    cursor: pointer;
    transition: all 0.6s;
  }

  #start {
    background-color: green;
  }

  #stop {
    background-color: rgb(151, 15, 15);
  }

  #app {
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
    text-align: left;
    color: #ffffff;
    font-size: 16px;
    position: relative;
    font-family: "PT Serif", serif;
    font-weight: 400;
    font-style: normal;
  }

  body {
    background-color: #222a39;
    margin-bottom: 10px;
  }
</style>
