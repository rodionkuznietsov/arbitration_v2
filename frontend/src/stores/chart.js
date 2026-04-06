import { LineSeries } from "lightweight-charts";
import { defineStore } from "pinia";
import { computed, effectScope, watch } from "vue";

export const useChartStore = defineStore('chart', {
    state: () => ({
        swapActive: null,
        legend: null,
        chart: null,
        inSpreadSeries: null,
        outSpreadSeries: null,
        scope: null,
        
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

        createSeries() {
            if (!this.chart) return

            if (this.inSpreadSeries != null && this.inPriceLine != null) {
                this.chart.removeSeries(this.inSpreadSeries)
                this.inSpreadSeries.removePriceLine(this.inPriceLine)
                this.inSpreadSeries = null
                this.inPriceLine = null
            }

            if (this.outSpreadSeries != null && this.outPriceLine != null) {
                this.chart.removeSeries(this.outSpreadSeries)
                this.outSpreadSeries.removePriceLine(this.outPriceLine)
                this.outPriceLine = null
            }

            const lastLongValue = computed(() => {
                return this.swapActive ? this.lastShortLine : this.lastLongLine
            })

            const lastShortValue = computed(() => {
                return this.swapActive ? this.lastLongLine : this.lastShortLine
            })

            this.inSpreadSeries = this.chart.addSeries(LineSeries, this.inSeriesOptions)
            this.inSpreadSeries.applyOptions({
                priceFormat: {
                    type: 'percent',
                    precision: this.precision,
                }
            })

            this.outSpreadSeries = this.chart.addSeries(LineSeries, this.outSeriesOptions)
            this.outSpreadSeries.applyOptions({
                priceFormat: {
                    type: 'percent',
                    precision: this.precision,
                }
            })

            this.inPriceLine = this.inSpreadSeries.createPriceLine(this.inPriceLine)
            this.outPriceLine = this.outSpreadSeries.createPriceLine(this.outPriceLine)

            this.scope = effectScope()
            
            this.scope.run(() => {
                watch(
                    (lastLongValue, lastShortValue), 
                    () => {
                        if (this.lastLongLine.time) {
                            this.inSpreadSeries.update(this.lastLongLine)
                            this.inPriceLine.applyOptions({
                                price: lastLongValue.value.value,
                                color: '#2EBD85',
                                title: `${lastLongValue.value.value.toFixed(5)} In %`,
                            })
                            this.chart.timeScale().scrollToRealTime()
                        }

                        if (this.lastShortLine.time) {
                            this.outSpreadSeries.update(this.lastShortLine)
                            this.outPriceLine.applyOptions({
                                price: lastShortValue.value.value,
                                color: '#F6465D',
                                title: `${lastShortValue.value.value.toFixed(5)} Out %`,
                                minMove: this.minMove                          
                            })
                            this.chart.timeScale().scrollToRealTime()
                        }
                    }
                )

                watch(
                    () => [this.linesLongHistory.length, this.linesShortHistory.length],
                    ([longLen, shortLen]) => {
                        if (longLen > 0 && shortLen > 0) {
                            this.inSpreadSeries.setData(this.linesLongHistory)
                            this.outSpreadSeries.setData(this.linesShortHistory)
                        }
                    }
                )
            })
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