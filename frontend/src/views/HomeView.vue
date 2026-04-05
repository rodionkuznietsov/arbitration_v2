<script setup>
    import {onMounted, ref } from 'vue'
    import FormCombobox from '../components/FormCombobox.vue';
    import OrderBook from '../components/OrderBook.vue';
    import AppHeader from '../components/AppHeader.vue'
    import { useWebsocketStore } from '@/stores/websocket';
    import { useUserState } from '@/stores/user_state';
    import { useChartStore } from '@/stores/chart';
    import { useOrderBookStore } from '@/stores/orderbook';

    const ws = useWebsocketStore()
    const userState = useUserState()

    const longExchange = ref("Нет доступных бирж")
    const shortExchange = ref("Нет доступных бирж")
    
    const exchanges = ref(null)
    onMounted(async () => {
      const response = await fetch('https://unfarming-untethered-flynn.ngrok-free.dev/api/exchanges/available')
      const data = await response.json()
      exchanges.value = data.exchanges.map(exchange => exchange.name)
      longExchange.value = exchanges.value[0]
      shortExchange.value = exchanges.value[1]
    })

    const market_types = ["Спот", "Фьючерс"]

    const orderBook = ref(null)
    const orderBookStore = useOrderBookStore()

    // Данные с полей
    const longOrderType = ref("Спот")
    const shortOrderType = ref("Спот")
    const ticker = ref("BTC")
    const chartStore = useChartStore()

    function filterInput(event) {
      ticker.value = event.target.value.replace(/[^a-zA-Z0-9]/g, '').toUpperCase()
    }

    async function start() {
      await ws.connect("wss://unfarming-untethered-flynn.ngrok-free.dev/ws")

      if (ws.socket.readyState == 3) {
        userState.changeStatus('warning')
        return
      }

      if (userState.botWorking) {
        return
      }

      userState.set_data(
        ticker.value, 
        longExchange.value, 
        shortExchange.value,
        longOrderType.value,
        shortOrderType.value,
      )
      userState.botWorking = true
      chartStore.finished = false

      orderBook.value.start()
      setTimeout(() => {
        chartStore.longExchangeLogo = orderBookStore.longExchangeLogo
        chartStore.shortExchangeLogo = orderBookStore.shortExchangeLogo
      }, 50)

      userState.changeStatus('online')
      chartStore.longExchange = userState.longExchange
      chartStore.shortExchange = userState.shortExchange
    }    
    
    async function stop() {
      userState.changeStatus('offline')

      await new Promise(resolve => {
        ws.disconnect()
        resolve()
      })
      userState.botWorking = false
      orderBook.value.stop() 
      orderBookStore.clearValues()
      chartStore.finished = true
      chartStore.clearValues()
    }
</script>

<template>
    <AppHeader/>

    <div class="scroll">
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
            <button class="default-btn" id="start" @click="start">Старт</button>
            <button class="default-btn" id="stop" @click="stop">Стоп</button>
          </div>

        <OrderBook ref="orderBook"/>
      </footer>
    </div>
</template>