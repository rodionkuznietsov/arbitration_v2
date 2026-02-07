<script setup>
    import {ref } from 'vue'
    import FormCombobox from '../components/FormCombobox.vue';
    import OrderBook from '../components/OrderBook.vue';
    import AppHeader from '../components/AppHeader.vue'

    const exchanges = ["Bybit", "Binance", "KuCoin", "BinX", "Mexc", "Gate", "Lbank"]
    const market_types = ["Спот", "Фьючерс"]
    const isWarning = ref(false)

    const orderBook = ref(null)
    const header = ref(null)

    // Данные с полей
    const longExchange = ref("Binance")
    const longOrderType = ref("Спот")
    const shortExchange = ref("Bybit")
    const shortOrderType = ref("Спот")
    const ticker = ref("BTC")

    function filterInput(event) {
    ticker.value = event.target.value.replace(/[^a-zA-Z0-9]/g, '').toUpperCase()
    }

    function start() {
    const data = {
        exchanges: {
        longExchange: longExchange.value,
        shortExchange: shortExchange.value,
        },
        types: {
        longType: longOrderType.value,
        shortType: shortOrderType.value,
        },
        ticker: ticker
    }

    orderBook.value.exchanges(data)
    orderBook.value.show()  
    orderBook.value.start()

    setTimeout(() => {
        if (isWarning.value) {
        header.value.change_work_status("warning")
        } else {
        header.value.change_work_status("online")
        }
    }, 10)
    }    
    
    function stop() {
        orderBook.value.stop()
        if (header.value) {
        header.value.change_work_status('offline')
        }
    }
</script>

<template>
    <AppHeader ref="header"/>

    <form id="form">
      <div id="form-column">
        <div class="form-group">
          <label for="ticker" id="form_label">Тикер (BTC):</label>
          <input id="ticker" name="ticker" type="text" value="BTC" class="form_input" v-model="ticker" @input="filterInput">
        </div>

        <div class="form-group">
          <div id="form_label-with_icon">
            <label for="order" id="form_label">Лонг:</label>
            <img src="../assets/icons/up.svg" alt="" draggable="false">
          </div>
          <FormCombobox v-model="longExchange" :options="exchanges"/>
        </div>

        <div class="form-group">
          <FormCombobox v-model="longOrderType" :options="market_types"/>
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
            <img class="img_reverse" src="../assets/icons/up.svg" alt="" draggable="false">
          </div>
          <FormCombobox v-model="shortExchange" :options="exchanges"/>
        </div>

        <div class="form-group">
          <FormCombobox v-model="shortOrderType" :options="market_types"/>
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

      <OrderBook ref="orderBook" v-model="isWarning"/>
    </footer>
</template>