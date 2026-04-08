<script setup>
    import {onMounted, ref } from 'vue'
    import FormCombobox from '../components/FormCombobox.vue';
    import OrderBook from '../components/OrderBook.vue';
    import AppHeader from '../components/AppHeader.vue'
    import { useWebsocketStore } from '@/stores/websocket';
    import { useUserState } from '@/stores/user_state';
    import { useChartStore } from '@/stores/chart';
    import { useOrderBookStore } from '@/stores/orderbook';
    import { useAuthStore } from '@/stores/auth';
    import AuthError from '@/components/AuthError.vue';
    import { API_URL, WEBSOCKET_URL } from '@/config';
    import { useTgStore } from '@/stores/tg';
    import { useHomeStore } from '@/stores/home';

    const authStore = useAuthStore()
    const ws = useWebsocketStore()
    const userState = useUserState()
    const tgStore = useTgStore()
    const homeStore = useHomeStore()

    onMounted(async () => {
      try {
        const response = await fetch(`${API_URL}/exchanges/available`)
        const data = await response.json()

        userState.exchanges = data.exchanges
        homeStore.longExchange = userState.exchanges[0] ? userState.exchanges[0].name : homeStore.longExchange
        homeStore.shortExchange = userState.exchanges[1] ? userState.exchanges[1].name : homeStore.longExchange
      } catch(err) {
        if (tgStore.tgObject) {
          tgStore.tgObject.showAlert("Системный сбой. Пожайлуйста сообщите об этом в службу поддержки.", () => {
            console.log("")
          })
        }
      }
    })

    const market_types = ["Спот", "Фьючерс"]

    const orderBook = ref(null)
    const orderBookStore = useOrderBookStore()
    const userStore = useUserState()

    // Данные с полей
    const longOrderType = ref("Спот")
    const shortOrderType = ref("Спот")
    const chartStore = useChartStore()

    function filterInput(event) {
      userStore.symbol = event.target.value.replace(/[^a-zA-Z0-9]/g, '').toUpperCase()
    }

    async function start() {
      await ws.connect(`${WEBSOCKET_URL}`)

      if (ws.socket.readyState == 3) {
        userState.changeStatus('warning')
        return
      }

      userState.set_data(
        userStore.symbol, 
        homeStore.longExchange, 
        homeStore.shortExchange,
        longOrderType.value,
        shortOrderType.value,
      )
      chartStore.finished = false

      orderBook.value.start()
      setTimeout(() => {
        chartStore.longExchangeLogo = orderBookStore.longExchangeLogo
        chartStore.shortExchangeLogo = orderBookStore.shortExchangeLogo
      }, 50)

      chartStore.longExchange = userState.longExchange
      chartStore.shortExchange = userState.shortExchange
    }    
    
    async function stop() {
      await new Promise(resolve => {
        ws.disconnect()
        resolve()
      })
      orderBook.value.stop() 
      orderBookStore.clearValues()
      chartStore.finished = true
      chartStore.clearValues()
    }
</script>

<template>
    <div class="page" v-if="authStore.success">
      <AppHeader />
      <div class="scroll">
        <form id="form">
          <div id="form-column">
            <div class="form-group">
              <label for="ticker" id="form_label">Тикер (BTC):</label>
              <input id="ticker" name="ticker" type="text" value="BTC" class="form_input" v-model="userStore.symbol" @input="filterInput">
            </div>

            <div class="form-group">
              <div id="form_label-with_icon">
                <label for="order" id="form_label">Лонг:</label>
                <img src="../assets/icons/up.svg" alt="" draggable="false">
              </div>
              <FormCombobox v-model="homeStore.longExchange" :options="userState.exchanges.map(c => c.name)"/>
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
              <FormCombobox v-model="homeStore.shortExchange" :options="userState.exchanges.map(c => c.name)"/>
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
    </div>

    <AuthError v-else/>
</template>