<script setup>
    import { useChartStore } from '@/stores/chart';
    import { useOrderBookStore } from '@/stores/orderbook';
    import { useUserState } from '@/stores/user_state';
    import { useWebsocketStore } from '@/stores/websocket';
    import { createChart, CrosshairMode, LineSeries } from 'lightweight-charts';
    import { computed, onActivated, onDeactivated, ref } from 'vue';
    import exchangeIconTrue from '../assets/icons/exchange_true.svg'
    import exchangeIconFalse from '../assets/icons/exchange_false.svg'

    const userStateStore = useUserState()
    const orderBookStore = useOrderBookStore()
    const isHovered = ref(false)

    const ws = useWebsocketStore()
    let unsubscribe

    let chart;
    const container = ref(null)

    const chartStore = useChartStore()
    let updateInterval

    const swapActive = ref(false)
    let legend = null

    const lastLongPriceSwapped = computed(() => {
        return swapActive.value ? orderBookStore.shortLastPrice : orderBookStore.longLastPrice
    })

    const lastShortPriceSwapped = computed(() => {
        return swapActive.value ? orderBookStore.longLastPrice : orderBookStore.shortLastPrice
    })

    const lastLongValue = computed(() => {
        return swapActive.value ? chartStore.lastShortLine : chartStore.lastLongLine
    })

    const lastShortValue = computed(() => {
        return swapActive.value ? chartStore.lastLongLine : chartStore.lastShortLine
    })

    let inSpreadSeries
    let outSpreadSeries
    let inPriceLine
    let outPriceLine

    onActivated(() => {
        unsubscribe = ws.subscribe(userStateStore.ticker, 'lines_history', userStateStore.longExchange, userStateStore.shortExchange, (result) => {                        
            const lines = result?.lines
            if (lines) {
                const long = lines.long

                const tempLongHistory = long ? long.map(line => ({
                    time: Math.floor(new Date(line.time).getTime() / 1000),
                    value: parseFloat(line.value)
                })) : {}

                const short = lines.short
                const tempShortHistory = short ? short.map(line => ({
                    time: Math.floor(new Date(line.time).getTime() / 1000),
                    value: parseFloat(line.value)
                })) : {}
                chartStore.linesLongHistory = tempLongHistory
                chartStore.linesShortHistory = tempShortHistory 
            }

            const events = result?.events
            if (events) {
                const updateLine = events?.update_line
                if (updateLine) {
                    const long = updateLine.long
                    const long_time = long?.time
                    const short = updateLine.short
                    const short_time = short?.time

                    if (long_time) {
                        chartStore.lastLongLine = {
                            time: Math.floor(new Date(long.time).getTime() / 1000),
                            value: parseFloat(long.value)
                        }
                    }

                    if (short_time) {
                        chartStore.lastShortLine = {
                            time: Math.floor(new Date(short.time).getTime() / 1000),
                            value: parseFloat(short.value)
                        }
                    }
                }
            }
        })

        const chartOptions = {
            width: container.value.clientWidth,
            height: container.value.clientHeight,

            timeScale: {
                timeVisible: true,
                secondsVisible: false,
            },
                    
            layout: {
                background: { color: '#ffffff', type: 'solid' },
                textColor: '#1f1f1f',
            },
            crosshair: {
                mode: CrosshairMode.Normal,
            },
            grid: {
                vertLines: {
                    color: '#efefef'
                },
                horzLines: {
                    color: '#efefef',
                },
            },
        }
        
        chart = createChart(container.value, chartOptions);
        
        legend = document.createElement('div')
        legend.style = `position: absolute; left: 10px; top: 100px; z-index: 10000; font-size: 14px;`
        legend.style.color = '#1f1f1f'
        legend.style.userSelect = 'none'
        legend.style.pointerEvents = 'none'
        container.value.appendChild(legend)

        legend.innerHTML = `<div><span>Оборот за 24 часа:</span><div style="text-transform: capitalize;">` + chartStore.longExchange + `: 0.0K</div><div style="text-transform: capitalize;">`+ chartStore.shortExchange +`: 0.0K</div></div>`
        createSeries(chart)

        chart.timeScale().applyOptions({
            lockVisibleTimeRangeOnResize: false,
            borderColor: '#dfdede',
            borderVisible: true,
            tickMarkFormatter: (time, tickMarkType, locale) => {
                const date = new Date(time * 1000);
                return date.toLocaleTimeString(locale, { hour: '2-digit', minute: '2-digit' });
            },
        })

        window.addEventListener('resize', () => {
            if (chart) {
                chart.resize(
                    window.innerWidth - 75,
                    window.innerHeight - 17
                )
                chart.timeScale().fitContent()
            }
        })
    })

    onDeactivated(() => {
        if (updateInterval) {
            clearInterval(updateInterval)
            updateInterval = undefined
        }

        if (chartStore.finished) {
            if (unsubscribe) {
                unsubscribe()
                unsubscribe = null
            }
        } 

        if (chart) {
            chart.remove();
            chart = null;
            inSpreadSeries = null
            outSpreadSeries = null
        }

        if (legend) {
            legend.remove()
            legend = null
        }

        window.removeEventListener('resize', () => {

        })
    })

    function createSeries(chart) {
        if (!chart) return
        if (inSpreadSeries) {
            chart.removeSeries(inSpreadSeries)
            inSpreadSeries = null
        }

        if (outSpreadSeries) {
            chart.removeSeries(outSpreadSeries)
            outSpreadSeries = null
        }

        inSpreadSeries = chart.addSeries(LineSeries, chartStore.inSeriesOptions)
        outSpreadSeries = chart.addSeries(LineSeries, chartStore.outSeriesOptions)

        inPriceLine = inSpreadSeries.createPriceLine(chartStore.inPriceLine)
        outPriceLine = outSpreadSeries.createPriceLine(chartStore.outPriceLine)

        updateInterval = setInterval(() => {
            if (chartStore.lastLongLine) {
                inSpreadSeries.update(chartStore.lastLongLine)
                inPriceLine.applyOptions({
                    price: lastLongValue.value.value,
                })
                chart.timeScale().scrollToRealTime()
            }

            if (chartStore.lastShortLine) {
                outSpreadSeries.update(chartStore.lastShortLine)
                outPriceLine.applyOptions({
                    price: lastShortValue.value.value,
                    color: '#F6465D'
                })
                chart.timeScale().scrollToRealTime()
            }
        }, 0)

        if (!swapActive.value) {
            setTimeout(() => {
                if (chartStore.linesLongHistory) {
                    inSpreadSeries.setData(chartStore.linesLongHistory)
                }
                if (chartStore.linesShortHistory) {
                    outSpreadSeries.setData(chartStore.linesShortHistory)
                }
                chart.timeScale().fitContent()
            }, 50)
        }
    }

    function swapExchange() {
        const tempLong = chartStore.longExchange
        const tempShort = chartStore.shortExchange
        const tempLongLogo = chartStore.longExchangeLogo
        const tempShortLogo = chartStore.shortExchangeLogo

        chartStore.longExchange = tempShort
        chartStore.shortExchange = tempLong
        chartStore.longExchangeLogo = tempShortLogo
        chartStore.shortExchangeLogo = tempLongLogo

        swapActive.value = !swapActive.value
        legend.innerHTML = `<div><span>Оборот за 24 часа:</span><div style="text-transform: capitalize;">` + chartStore.longExchange + `: 0.0K</div><div style="text-transform: capitalize;">`+ chartStore.shortExchange +`: 0.0K</div></div>`

        createSeries()

        changeLineSeriesColor()
    }

    function changeLineSeriesColor() {
        swapActive.value ? inSpreadSeries.applyOptions({
            color: '#F6465D'
        }) : inSpreadSeries.applyOptions({
            color: '#2EBD85'
        })

        !swapActive.value ? outSpreadSeries.applyOptions({
            color: '#F6465D'
        }) : outSpreadSeries.applyOptions({
            color: '#2EBD85'
        })
    }
</script>

<template>
    <div class="chart-container">
        <div class="toolbar">
            <div class="chart-exchanges">
                <div class="long-ex">
                    <div>
                        <span class="exchange-name">{{ chartStore.longExchange }}</span>
                        <img class="exchange-icon" :src="chartStore.longExchangeLogo ? chartStore.longExchangeLogo : ''">
                    </div>
                    <div class="price-type-icon">
                        <img class="market_type long" src="../assets/icons/long.svg">
                        <span class="longPrice">{{ lastLongPriceSwapped }}</span>
                    </div>
                </div>
                <div class="short-ex">
                    <div>
                        <span class="exchange-name">{{ chartStore.shortExchange }}</span>
                        <img class="exchange-icon" :src="chartStore.shortExchangeLogo ? chartStore.shortExchangeLogo : ''">
                        <!-- <span>Оборот за 24 часа: $675,95K</span> -->
                    </div>
                    <div class="price-type-icon">
                        <span class="shortPrice">{{ lastShortPriceSwapped }}</span>
                        <img class="market_type short" src="../assets/icons/short.svg">
                    </div>
                </div>
            </div>
        </div>
        <div class="left-menu">
            <img @click="swapExchange()" class="item" :src="
            isHovered ? exchangeIconTrue : exchangeIconFalse"
            @mouseenter="isHovered = true"
            @mouseleave="isHovered = false"
            title="Поменять направления"
            />
        </div>
        <div class="chart" ref="container" id="chart">
            <div class="ticker">{{ userStateStore.ticker }}</div>
        </div>
    </div>
</template>

<style scoped>
    .chart {
        display: block;
        height: 98vh;
        box-sizing: border-box;
        position: relative;
        left: 50px;
        margin-right: 50px;
        bottom: 50px;
    }

    .chart-container {
        width: 100%;
        box-sizing: border-box;
        position: absolute;
        height: 100vh;
        overflow: hidden;
    }

    .ticker {
        display: flex;
        z-index: 100000000;
        position: fixed;
        color: var(--default-chart-ticker-color);
        justify-content: center;
        align-items: center;
        font-size: 20px;
        width: 100%;
        height: 100%;
        left: 0;
        top: 0;
        text-transform: uppercase;
        font-weight: 600;
        pointer-events: none;
        user-select: none;
    }

    .toolbar {
        top: 0;
        display: block;
        padding: var(--default-padding);
        position: fixed;
        z-index: 1000000000;
        background-color: var(--basic-Bg);
        width: 100%;
        height: 55px;
        border-bottom: 1px solid var(--color-chart-border-bottom);
        box-sizing: border-box;
        right: 0;
        font-size: var(--default-font-size);
    }

    .item {
        width: var(--default-icon-size);
        height: var(--default-icon-size);
        background-color: var(--default-input-bg);
        padding: var(--default-padding);
        border-radius: var(--default-border-radius);
        margin-top: 10px;
        margin-right: 10px;
    }

    .item:hover {
        cursor: pointer;
    }

    .market_type {
        width: var(--default-icon-size);
        height: var(--default-icon-size);
        border-radius: 50px;
        display: inline;
        position: sticky;
        top: 10px;
    }

    .long {
        background-color: var(--color-success);
        margin-right: 5px;
    }

    .short {
        background-color: var(--color-error);
        margin-left: 5px;
    }

    .price-type-icon {
        display: flex;
        align-items: center;
    }

    .exchange-name {
        text-transform: capitalize;
    }
    
    .exchange-icon {
        width: var(--default-icon-size);
        height: var(--default-icon-size);
        display: inline;
        position: sticky;
        top: 10px;
        margin-left: 5px;
    }

    .longPrice, .shortPrice {
        font-weight: 600;
    }

    .longPrice {
        color: var(--color-success);
    }

    .long-ex {
        display: flex;
        flex-direction: column;
        align-items: flex-start;
    }

    .shortPrice {
        color: var(--color-error);
    }

    .short-ex {
        display: flex;
        flex-direction: column;
        align-items: flex-end;
    }

    .left-menu {
        display: flex;
        justify-content: center;
        height: 85vh;
        top: 55px;
        z-index: 1000000000;
        position: fixed;
        border-right: 1px solid var(--color-chart-border-bottom);
        background-color: var(--basic-Bg);
        width: 50px;
        gap: 10px;
    }

    .intervals {
        display: flex;
    }

    .intervals::after {
        content: '';
        display: flex;
        margin-right: 10px;
        align-items: center;
        width: 1px;
        background-color: var(--default-font-color);
        font-size: var(--default-font-size);
    }

    .toolbar-button {
        background-color: var(--default-input-color);
        color: var(--default-font-color);
        border: none;
        margin-right: 10px;
        cursor: pointer;
        padding: var(--default-border-radius);
        border-radius: 4px;
        font-size: var(--chart-button-font-size);
        width: 50px;
        height: 28px;
    }

    .toolbar-button:hover {
        background-color: #303c517a;
    }

    .chart-exchanges {
        display: flex;
        gap: var(--default-gap);
        justify-content: space-between;
    }

    .title_bg {
        position: absolute;
        top: 50%;
        left: 50%;
        transform: translate(-50%, -50%);
        font-size: 18px;
        color: var(--chart-title-font-color);
        pointer-events: none;
        z-index: 1000000000;
        font-weight: bold;
    }
</style>