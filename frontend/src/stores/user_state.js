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
        set_init_data(data) {
            this.isBotRunning = data.isBotRunning
            this.symbol = data.symbol

            this.longExchange = data.longExchange
            this.longOrderType = data.longOrderType
            
            this.shortExchange = data.shortExchange
            this.shortOrderType = data.shortOrderType

            this.currentStatus = data.status
        },

        set_exchange() {
            console.log("")
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