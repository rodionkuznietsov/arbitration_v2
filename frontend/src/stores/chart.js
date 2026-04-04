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

        precision: 100_000,
        minMove: 0.0000000000001,
        inSeriesOptions: {
            color: '#2EBD85',
            lastValueVisible: false,
            priceLineVisible: false,
            lineWidth: 2,
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
        priceFormat(value) {
            return new Intl.NumberFormat(
                "en-US", {
                    minimumFractionDigits: 1,
                    maximumFractionDigits: 9,
                }
            ).format(value)
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