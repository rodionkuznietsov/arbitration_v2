import { defineStore } from "pinia";

export const useUserState = defineStore('userState', {
    state: () => ({
        ticker: null,
        longExchange: null,
        shortExchange: null,
        longOrderType: null,
        shortOrderType: null,
        currentStatus: 'offline',
        candles_history: []
    }),

    actions: {
        set_data(
            ticker, 
            longExchange,
            shortExchange,
            longOrderType,
            shortOrderType,
        ) {
            this.ticker = ticker.toLowerCase()
            this.longExchange = longExchange.toLowerCase()
            this.shortExchange = shortExchange.toLowerCase()
            this.longOrderType = longOrderType.toLowerCase()
            this.shortOrderType = shortOrderType.toLowerCase()
        },

        changeStatus(status) {
            this.currentStatus = status
        },

        get_data() {
            return {
                ticker: this.ticker,
                longExchange: this.longExchange,
                shortExchange: this.shortExchange,
                longOrderType: this.longOrderType,
                shortOrderType: this.shortOrderType,
            }
        },

        clearValues() {
            this.ticker = null
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