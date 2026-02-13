import { defineStore } from "pinia";
import { computed } from "vue";

export const useOrderBookStore = defineStore('orderbook', {
    state: () => ({
        ticker: null,

        longExchange: null,
        longExchangeLogo: null, 
        longOrderType: null,
        longArrow: null,
        longAsks: [],
        longBids: [],
        longLastPrice: 0.0,
        longFirstAskPrice: 0.0,
        longFirstBidPrice: 0.0,
        
        shortExchange: null,
        shortExchangeLogo: null,
        shortOrderType: null,
        shortArrow: null,
        shortBids: [],
        shortAsks: [],
        shortLastPrice: 0.0,
        shortFirstAskPrice: 0.0,
        shortFirstBidPrice: 0.0,
    }),

    actions: {
        updateHeader(
            ticker, 
            longEx, 
            longOrderType,
            shortEx,
            shortOrderType
        ) {
            this.ticker = ticker

            this.longExchange = longEx
            this.longExchangeLogo = `../assets/icons/${this.longExchange.toLowerCase()}_logo.svg`
            this.longOrderType = longOrderType
            this.longArrow = computed(() => {
                if (this.longLastPrice == this.longFirstAskPrice) {
                    return '⬆'
                } else if (this.longLastPrice == this.longFirstBidPrice) {
                    return '⬇'
                } 
                return '⬆'
            })

            this.shortExchange = shortEx
            this.shortExchangeLogo = `../assets/icons/${this.shortExchange.toLowerCase()}_logo.svg`
            this.shortOrderType = shortOrderType
            this.shortArrow = computed(() => {
                if (this.shortLastPrice == this.shortFirstAskPrice) {
                    return '⬆'
                } else if (this.shortLastPrice == this.shortFirstBidPrice) {
                    return '⬇'
                } 
                return '⬆'
            })
        },

        updateData(books) {
            this.longLastPrice = books.long?.last_price
            this.longFirstAskPrice = books.long?.a?.[0]?.[0];
            this.longFirstBidPrice = books.long?.b?.[0]?.[0];

            if (books.long?.a) {
                this.longAsks.length = 0
                this.longAsks.push(...books.long.a.map(x => ({
                    price: x[0],
                    volume: x[1],
                })))
            }

            if (books.long?.b) {
                this.longBids.length = 0
                this.longBids.push(...books.long.b.map(x => ({
                    price: x[0],
                    volume: x[1],
                })))
            }

            if (books.short?.a) {
                this.shortAsks.length = 0
                this.shortAsks.push(...books.short.a.map(x => ({
                    price: x[0],
                    volume: x[1],
                })))
            }

            if (books.short?.b) {
                this.shortBids.length = 0
                this.shortBids.push(...books.short.b.map(x => ({
                    price: x[0],
                    volume: x[1],
                })))
            }

            this.shortLastPrice = books.short?.last_price
            this.shortFirstAskPrice = books.short?.a?.[0]?.[0];
            this.shortFirstBidPrice = books.short?.b?.[0]?.[0];
        },

        clearValues() {
            this.ticker = null

            this.longExchange = null
            this.longArrow = null
            this.longExchangeLogo = null
            this.longOrderType = null

            this.shortExchange = null
            this.shortArrow = null
            this.shortExchangeLogo = null
            this.shortOrderType = null
        }     
    }
})