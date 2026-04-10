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
  import { useHomeStore } from './stores/home';
  import { useOrderBookStore } from './stores/orderbook';
  import FetchErrorImg from './assets/img/fetch_error.png'
  import Error401Img from './assets/img/401.png'
  
  const authStore = useAuthStore()
  const userStateStore = useUserState()
  const orderBookStore = useOrderBookStore()
  const tgStore = useTgStore()
  const homeStore = useHomeStore()

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
          
          if (event_data.type == "log") {
            try {
              // userStateStore.isBotRunning = event_data.payload.isBotRunning
              userStateStore.symbol = event_data.payload.symbol.replace("USDT", ""),
              homeStore.longExchange = event_data.payload.longExchange,
              userStateStore.longOrderType = event_data.payload.longOrderType,
              homeStore.shortExchange = event_data.payload.shortExchange,
              userStateStore.shortOrderType = event_data.payload.shortOrderType

              userStateStore.logs.push({
                event: event_data.payload.event,
                symbol: event_data.payload.symbol,
                long_exchange: event_data.payload.long_exchange,
                short_exchange: event_data.payload.short_exchange,
                timestamp: event_data.timestamp
              })
              userStateStore.logs.sort((a, b) => b.timestamp - a.timestamp) // DESC
            } catch(err) {
              tg.showAlert("Ошибка загрузки страницы", () => {
                console.log("Пользователь закрыл alert")
              });
            }
          } else if (event_data.type == "exchange") {
            if (event_data.payload.event == "update_exchange") {
              if (!event_data.payload.is_available) {
                // Удаляем биржу
                userStateStore.exchanges = userStateStore.exchanges.filter(
                  ex => ex.name !== event_data.payload.exchange_name
                )

                if (userStateStore.exchanges.length > 0) {
                  homeStore.longExchange = userStateStore.exchanges[0].name
                  homeStore.shortExchange = userStateStore.exchanges[0].name
                } else {
                  homeStore.longExchange = "Нет доступных бирж"
                  homeStore.shortExchange = "Нет доступных бирж"
                }
              } else {
                const index = userStateStore.exchanges.findIndex(
                  ex => ex.name === event_data.payload.exchange_name
                )

                // Добавляем биржу
                if (index === -1) {
                  userStateStore.exchanges.push({
                    id: userStateStore.exchanges.length + 1,
                    name: event_data.payload.exchange_name,
                    is_available: true
                  })
                }

                if (userStateStore.exchanges.length > 1) {
                  homeStore.longExchange = userStateStore.exchanges[1].name
                  homeStore.shortExchange = userStateStore.exchanges[0].name
                } else {
                  homeStore.longExchange = userStateStore.exchanges[0].name
                  homeStore.shortExchange = userStateStore.exchanges[0].name
                }
              }
            } else if (event_data.payload.event == "add_exchange") {
              // Добавляем биржу
              userStateStore.exchanges.push({
                id: userStateStore.exchanges.length + 1,
                name: event_data.payload.exchange_name,
                is_available: true
              })
            }
          } else if (event_data.type == "websocket") {
            userStateStore.changeStatus(event_data.payload.status)
            userStateStore.isBotRunning = event_data.payload.isBotRunning

            // Устанавливаем валидные данные для отображения стакана
            if (userStateStore.isBotRunning) {
              orderBookStore.updateHeader(
                event_data.payload.symbol,
                event_data.payload.longExchange,
                event_data.payload.longOrderType,
                event_data.payload.shortExchange,
                event_data.payload.shortOrderType
              )
            }
            
            if (event_data.ws_data.channel == "order_book") {
              orderBookStore.longLastPrice = event_data.ws_data.result.data.order_book.long.last_price
              orderBookStore.shortLastPrice = event_data.ws_data.result.data.order_book.short.last_price
              
              orderBookStore.longAsks = event_data.ws_data.result.data.order_book.long.asks
              orderBookStore.longBids = event_data.ws_data.result.data.order_book.long.bids
              
              orderBookStore.shortAsks = event_data.ws_data.result.data.order_book.short.asks
              orderBookStore.shortBids = event_data.ws_data.result.data.order_book.short.bids
            }
          } else if (event_data.type == "user_state") {
            if (userStateStore.isBotRunning) {
              orderBookStore.updateHeader(
                event_data.payload.symbol,
                event_data.payload.longExchange,
                event_data.payload.longOrderType,
                event_data.payload.shortExchange,
                event_data.payload.shortOrderType
              )
            }

            userStateStore.symbol = event_data.payload.symbol
            userStateStore.longExchange = event_data.payload.longExchange
            userStateStore.longOrderType = event_data.payload.longOrderType

            userStateStore.shortExchange = event_data.payload.shortExchange
            userStateStore.shortOrderType = event_data.payload.shortOrderType
          }
        }

        es.onerror = (event) => {
          console.log(event)
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

<style>
  @import url('https://fonts.googleapis.com/css2?family=PT+Serif:ital,wght@0,400;0,700;1,400;1,700&family=Vollkorn+SC:wght@400;600;700;900&display=swap');

  :root {
    --basic-Bg: #ffffff;
    --color-input-border: #b7bcc4;
    --color-popup: #333B47;
    --color-success: #16C784;
    --color-success-opacity-0_15: hsl(157 61% 46% / 0.15);
    --color-error: #EE8282;
    --color-error-opacity-0_15: hsl(352 91% 62% / 0.15);
    --default-font-color: #1f1f1f;
    --default-font-size: 16px;
    --default-padding: 8px;
    --default-icon-size: 16px;
    --default-border-radius: 8px;
    --default-input-color: #DFDFDF80;
    --default-chart-ticker-color: #DFDFDF80;
    --default-font: "Noto Sans", sans-serif;
    --default-input-bg: #DFDFDF80;
    --default-orderbook-bg: #DFDFDF80;
    --footer-margin-bottom: 50px;
    --default-gap: 10px;
    --default-margin-bottom: 60px;
    --default-bottom-px: 10px;

    /* AppMenu */
    --router-link-font-size: 16px;
    --wrapper-margin-right-left-px: 20px;
    --app_menu-bg-color: #DFDFDF80;

    /* Chart */
    --color-chart-border-left-menu-right: #dfdede;
    --chart-button-font-size: 10px;
    --chart-title-font-color: #DFDFDF80;
    --chart-padding-left-menu-px: 8px;
    --chart-bg-left-menu-color: #DFDFDF80;
    --chart-bg-header-color: #DFDFDF80;
    --color-chart-header-border-bottom: #dfdede;

    /* Latest Best Spread */
    --latest-best-spread-table-bg-color: #fb7b4d;
    --latest-best-spread-table-header-bg-color: #DFDFDF80;

    /* Logs */
    --log-table-bg-color: #DFDFDF80;
  }

  .page {
    width: 100%;
    height: 100%;
    position: absolute;
    left: 0;
    right: 0;
    top: 0;
    bottom: 0;
    overflow-y: auto;
    overflow-x: hidden;
  }

  .scroll {
    padding: var(--default-padding);
    box-sizing: border-box;
    margin-bottom: var(--default-margin-bottom);
  }

  .view_header {
      padding: var(--default-padding);
      font-size: var(--default-font-size);
  }

  #status {
    display: flex;
    gap: 10px;
    align-items: center;
  }
  
  #status_circle {
    width: var(--default-icon-size);
    height: var(--default-icon-size);
    border-radius: 50%;
  }

  @keyframes GlitchAnimation {
    from { opacity: 1; }
    to { opacity: 0; }
  }

  .online {
    background-color: var(--color-success);
    animation-name: GlitchAnimation;
    animation-duration: 1.6s;
    animation-iteration-count:  infinite;
  }

  .title {
      color: var(--default-font-color);
  }
 
  #form {
    display: grid;
    gap: 10px;
    width: 100%;
    box-sizing: border-box;
    justify-content: space-around;
    grid-template-columns: repeat(auto-fit, minmax(100px, 1fr));
    border-radius: var(--default-border-radius);
    padding: var(--default-padding);
    font-size: var(--default-font-size);
  }

  #form_label-with_icon {
    display: flex;
    flex-direction: row;
    align-items: center;
    justify-content: flex-start;
    gap: 5px;
  }

  #form_label-with_icon img {
    width: var(--default-icon-size);
    height: var(--default-icon-size);
  }

  .img_reverse {
    transform: rotate(60deg);
  }

  #form-column{
    display: flex;
    flex-direction: column;
  }

  .form_input {
    width: 100%;
    padding: var(--default-padding);
    border: none;
    color: var(--default-font-color);
    border-radius: var(--default-border-radius);
    font-size: var(--default-font-size);
    margin-top: 10px;
    background-color: var(--default-input-bg);
    outline: none;
    box-sizing: border-box;
    text-transform: uppercase;
  }

  .form-group {
    display: flex;
    flex-direction: column;
  }

  #footer {
    display: flex;
    flex-direction: column;
    margin-top: 10px;
    padding: var(--default-padding);
    overflow-y: visible;
    margin-bottom: var(--footer-margin-bottom);
  }

  #run_buttons {
    display: flex;
    gap: 10px;
  }

  .default-btn {
    flex: 1;
    font-size: var(--default-font-size);
    transition: all 0.25s;
    padding: var(--default-padding);
    color: var(--default-font-color);
    border-radius: var(--default-border-radius);
    border: none;
    outline: none;
  }

  .default-btn:hover {
    filter: opacity(75%);
    cursor: pointer;
    transition: all 0.25s;
  }

  .default-button-margin {
    margin-bottom: var(--default-margin-bottom);
  }

  #start, .long {
    background-color: var(--color-success);
  } 

  #stop, .short {
    background-color: var(--color-error);
  } 

  #app {
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
    text-align: left;
    color: var(--default-font-color);
    font-family: var(--default-font);
    font-weight: 400;
    font-style: normal;
  }

  .icon_svg {
    width: var(--default-icon-size);
    height: var(--default-icon-size);
    pointer-events: none;
    user-select: none;
  }

  .icon_svg:hover {
    cursor: pointer;
  }

  .svg_margin {
    margin-right: 10px;
  }

  .span-text {
    color: var(--default-font-color);
    text-transform: capitalize;
  }

  html, body {
    height: 100%;
    margin: 0;
  }

  body {
    background-color: var(--basic-Bg);
    display: flex;
    justify-content: center;
    align-items: center;
  }
</style>
