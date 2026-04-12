<script setup>
    import {onMounted, ref } from 'vue'
    import FormCombobox from '../components/FormCombobox.vue';
    import OrderBook from '../components/OrderBook.vue';
    import AppHeader from '../components/AppHeader.vue'
    import { useUserState } from '@/stores/user_state';
    import { useChartStore } from '@/stores/chart';
    import { useOrderBookStore } from '@/stores/orderbook';
    import { useAuthStore } from '@/stores/auth';
    import { API_URL } from '@/config';
    import { useConfigStore } from '@/stores/config';

    const authStore = useAuthStore()
    const userStore = useUserState()
    const configStore = useConfigStore()

    onMounted(async () => {
      try {
        const response = await fetch(`${API_URL}/exchanges/available`)
        const data = await response.json()

        configStore.exchanges = data.message.exchanges
      } catch(err) {
        console.log(err)
      }
    })

    const market_types = ["Спот", "Фьючерс"]

    const orderBook = ref(null)
    const orderBookStore = useOrderBookStore()

    const chartStore = useChartStore()

    function filterInput(event) {
      userStore.symbol = event.target.value.replace(/[^a-zA-Z0-9]/g, '').toUpperCase()
    }

    async function start() {
      chartStore.finished = false

      orderBook.value.start()
      setTimeout(() => {
        chartStore.longExchangeLogo = orderBookStore.longExchangeLogo
        chartStore.shortExchangeLogo = orderBookStore.shortExchangeLogo
      }, 50)

      chartStore.longExchange = userStore.longExchange
      chartStore.shortExchange = userStore.shortExchange
    }    
    
    async function stop() {
      orderBook.value.stop() 
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
              <FormCombobox v-model="userStore.longExchange" :values="configStore.exchanges" :event="'exchange_update'" :market_type="long"/>
            </div>

            <div class="form-group">
              <FormCombobox v-model="userStore.longOrderType" :values="market_types" />
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
              <FormCombobox v-model="userStore.shortExchange" :values="configStore.exchanges" :event="'exchange_update'" :market_type="'short'"/>
            </div>

            <div class="form-group">
              <FormCombobox v-model="userStore.shortOrderType" :values="market_types"/>
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
</template>