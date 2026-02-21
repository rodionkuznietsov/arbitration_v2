import { defineStore } from "pinia";

export const useChartStore = defineStore('chart', {
    state: () => ({
        finished: false,
        longExchange: null,
        linesLongHistory: [],
        longExchangeLogo: null,

        shortExchangeLogo: null,
        shortExchange: null,
        linesShortHistory: [],

        lastLongPrice: 0.0,
        lastShortPrice: 0.0,
        lastShortLine: {},
        lastLongLine: {},

        percision: 10000,
        minMove: 0.0000000000001
    }),
    actions: {
        clearValues() {
            this.longExchange = null
            this.shortExchange = null

            this.longExchangeLogo = null
            this.shortExchangeLogo = null

            this.lastLongLine = null
            this.lastShortLine = null

            this.lastLongPrice = null
            this.lastShortPrice = null

            this.linesLongHistory = []
            this.linesShortHistory = []
        }
    }
})