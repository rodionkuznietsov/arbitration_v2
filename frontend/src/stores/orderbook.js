import { defineStore } from "pinia";
import { computed } from "vue";

export const useOrderBookStore = defineStore('orderbook', {
    state: () => ({
        symbol: null,

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
        shortLastAskPrice: 0.0,

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
            this.longExchangeLogo = `../assets/icons/${this.longExchange.toLowerCase().replace(".", "_")}_logo.svg`
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
            this.shortExchangeLogo = `../assets/icons/${this.shortExchange.toLowerCase().replace(".", "_")}_logo.svg`
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

        updateData(result) {
            this.longLastPrice = result.long?.last_price
            this.longFirstAskPrice = result.long?.asks?.at(-1).price;
            this.longFirstBidPrice = result.long?.bids?.[0]?.price;

            if (result.long?.asks) {
                this.longAsks.length = 0
                this.longAsks = result.long.asks
            }

            if (result.long?.bids) {
                this.longBids.length = 0
                this.longBids = result.long.bids
            }

            if (result.short?.asks) {
                this.shortAsks.length = 0
                this.shortAsks = result.short.asks
            }

            if (result.short?.bids) {
                this.shortBids.length = 0
                this.shortBids = result.short.bids
            }

            this.shortLastPrice = result.short?.last_price
            this.shortFirstAskPrice = result.short?.asks?.at(-1)?.price;
            this.shortFirstBidPrice = result.short?.bids?.[0]?.price;
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

            this.longAsks = []
            this.shortAsks = []
            this.longBids = []
            this.shortBids = []

            this.longFirstAskPrice = 0.0
            this.shortFirstAskPrice = 0.0
            this.longFirstBidPrice = 0.0
            this.shortFirstBidPrice = 0.0

            this.longLastPrice = 0.0
            this.shortLastPrice = 0.0
        }     
    }
})