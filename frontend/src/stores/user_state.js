import { defineStore } from "pinia";

export const useUserState = defineStore('userState', {
    state: () => ({        
        // Параметры для данных из backend 
        isBotRunning: false,
        symbol: 'BTC',
        longOrderType: "Спот",
        shortOrderType: "Спот",
        
        logs: [],
        exchanges: {},

        longExchange: null,
        shortExchange: null,
        currentStatus: 'offline',
        bybit_api_key: null,
        bybit_api_secret: null
    }),

    actions: {
        set_data(
            symbol, 
            longExchange,
            shortExchange,
        ) {
            this.symbol = symbol.toLowerCase()
            this.longExchange = longExchange.toLowerCase()
            this.shortExchange = shortExchange.toLowerCase()
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