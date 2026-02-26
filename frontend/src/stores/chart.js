import { defineStore } from "pinia";

export const useChartStore = defineStore('chart', {
    state: () => ({
        finished: false,
        longExchange: null,
        linesLongHistory: [],
        longExchangeLogo: null,
        longVolume24hr: 0.0,

        shortExchangeLogo: null,
        shortExchange: null,
        shortVolume24hr: 0.0,
        linesShortHistory: [],

        lastLongPrice: 0.0,
        lastShortPrice: 0.0,
        lastShortLine: {},
        lastLongLine: {},

        percision: 1000,
        minMove: 0.0000000000001,
        inSeriesOptions: {
            color: '#2EBD85',
            lastValueVisible: false,
            priceLineVisible: false,
            lineWidth: 2
        },
        inPriceLine: {
            price: 0.0,
            color: '#2EBD85',
            title: 'In %',
        },
        outSeriesOptions: {
            color: '#F6465D',
            lastValueVisible: false,
            priceLineVisible: false,
            lineWidth: 2
        },
        outPriceLine: {
            price: 0.0,
            color: '#F6465D',
            title: 'Out %',
        },
    }),
    actions: {
        volume24hrFormat(value) {
            if (value >= 1_000_000_000) {
                return (value / 1_000_000_000).toFixed(2) + 'B';
            }

            if (value >= 1_000_000) {
                return (value / 1_000_000).toFixed(2) + 'M';
            }

            if (value >= 1_000) {
                return (value / 1_000).toFixed(2) + 'K'
            }

            if (value >= 0) {
                return value.toFixed(2) + '$'
            }
        },

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