<template>
  <div id="app">
    <router-view v-slot="{ Component }">
      <keep-alive>
        <component 
          :is="Component"
        />
      </keep-alive>
    </router-view>

    <footer id="footer" v-if="authStore.success">
      <AppMenu />
    </footer>
  </div>
</template>

<script setup>
  import { onMounted } from 'vue';
  import AppMenu from './components/AppMenu.vue';
  import { useAuthStore } from './stores/auth';
  import { API_URL } from './config';
  import { useUserState } from './stores/user_state';
  import { useTgStore } from './stores/tg';
  import { useOrderBookStore } from './stores/orderbook';
  import FetchErrorImg from './assets/img/fetch_error.png'
  import Error401Img from './assets/img/401.png'
  import { ws_handler } from './handlers/websocket.handler';
  import { log_handler } from './handlers/log.handler';
  import { useLogStore } from './stores/log.store';
  import { exchange_handler } from './handlers/exchange.handler';
  import { useConfigStore } from './stores/config';
  import { user_state_handler } from './handlers/user_state.handler';
  
  const authStore = useAuthStore()
  const userStateStore = useUserState()
  const orderBookStore = useOrderBookStore()
  const tgStore = useTgStore()
  const logStore = useLogStore()
  const configStore = useConfigStore()

  const handlers = {
    log: (data) => log_handler(data, tgStore, logStore),
    exchange: (data) => exchange_handler(data, configStore),
    websocket: (data) => ws_handler(data, userStateStore, orderBookStore),
    user_state: (data) => user_state_handler(data, userStateStore, orderBookStore, configStore),
    default: (data) => {
      console.warn("Не известное собитие", data.type)
    }
  }

  onMounted(async () => {
    // Авторизация в Telegram Web App
    if (window.Telegram && window.Telegram.WebApp) {
      const tg = window.Telegram.WebApp;
      try {
        tg.ready();

        tgStore.tgObject = tg

        const response = await fetch(`${API_URL}/telegram/bot/auth`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ initData: tg.initData })
        })

        if (!response.ok) {
          const errorData = await response.json();
          throw Object.assign(new Error(errorData.message || "Неизвестная ошибка"), {
            status: response.status || 500
          })
        }

        const data = await response.json()
        authStore.token = data.message.token_data.access_token
        authStore.tg_user_id = data.message.tg_user_id
        authStore.success = data.success

        // Подписываемся на слушения изменений
        const es = new EventSource(`${API_URL}/subscribe/events/${authStore.tg_user_id}`)

        es.onmessage = (event) => {
          const event_data = JSON.parse(event.data)

          const handler = handlers[event_data.type] || handlers.default
          handler(event_data)
        }

        es.onerror = (error) => {
          console.log(error)
        }

      } catch(err) {
        const img = document.createElement('img')

        if (err.status == 400) {
          img.src = Error401Img
        } else {
          img.src = FetchErrorImg
        }

        img.style.width = "100%"
        img.style.height = "100%"
        img.style.objectFit = 'cover'
        img.style.pointerEvents = 'none'
        img.style.userSelect = 'none'
        
        document.getElementById("app").appendChild(img)
      }
    }
  });

</script>

