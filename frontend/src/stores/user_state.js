import { defineStore } from "pinia";

export const useUserState = defineStore('userState', {
    state: () => ({        
        // Параметры для данных из backend 
        isBotRunning: false,
        symbol: null,
        
        longExchange: null,
        longOrderType: null,

        shortExchange: null,
        shortOrderType: null,

        currentStatus: null,
        bybit_api_key: null,
        bybit_api_secret: null
    }),

    actions: {
        set_init_data(payload, orderBookStore) {

            this.isBotRunning = payload.data.isBotRunning
            this.currentStatus = payload.data.status

            this.symbol = payload.bot_config.draft.symbol

            this.longExchange = payload.bot_config.draft.longExchange
            this.longOrderType = payload.bot_config.draft.longOrderType
            
            this.shortExchange = payload.bot_config.draft.shortExchange
            this.shortOrderType = payload.bot_config.draft.shortOrderType

            if (this.isBotRunning) {
                orderBookStore.updateHeader(
                    payload.bot_config.active.symbol,
                    payload.bot_config.active.longExchange,
                    payload.bot_config.active.longOrderType,
                    payload.bot_config.active.shortExchange,
                    payload.bot_config.active.shortOrderType
                )
            }
        },

        set_exchange(data) {
            alert(JSON.stringify(data))
        },

        changeStatus(status) {
            this.currentStatus = status
        },

        get_data() {
            return {
                symbol: this.symbol,
                longExchange: this.longExchange,
                shortExchange: this.shortExchange,
                longOrderType: this.longOrderType,
                shortOrderType: this.shortOrderType,
            }
        },

        clearValues() {
            this.symbol = null
            this.currentStatus = 'offline'

            this.longExchange = null
            this.shortExchange = null
            this.longOrderType = null
            this.shortOrderType = null

            this.longAsks = []
            this.shortAsks = []
            this.longBids = []
            this.shortBids = []

            this.longLastPrice = 0.0
            this.shortLastPrice = 0.0
            this.longFirstAskPrice = 0.0
            this.shortFirstAskPrice = 0.0
            this.longFirstBidPrice = 0.0
            this.shortFirstBidPrice = 0.0
        }
    }
})