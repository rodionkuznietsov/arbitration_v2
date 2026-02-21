<script setup>
    import { useChartStore } from '@/stores/chart';
import { useOrderBookStore } from '@/stores/orderbook';
    import { useUserState } from '@/stores/user_state';
    import { useWebsocketStore } from '@/stores/websocket';
    import { createChart, CrosshairMode, LineSeries } from 'lightweight-charts';
    import { onActivated, onDeactivated, ref } from 'vue';

    const userStateStore = useUserState()
    const orderBookStore = useOrderBookStore()

    const ws = useWebsocketStore()
    let unsubscribe

    let chart;
    const container = ref(null)

    const chartStore = useChartStore()
    let updateInterval

    onActivated(() => {
        unsubscribe = ws.subscribe(userStateStore.ticker, 'lines_history', userStateStore.longExchange, userStateStore.shortExchange, (result) => {
            console.log(result)
            
            const lines = result.lines
            if (lines) {
                const long = lines.long
                userStateStore.linesLongHistory = long.map(line => ({
                    time: Math.floor(new Date(line.time).getTime() / 1000),
                    value: parseFloat(line.value)
                }))

                const short = lines.short
                userStateStore.linesShortHistory = short.map(line => ({
                    time: Math.floor(new Date(line.time).getTime() / 1000),
                    value: parseFloat(line.value)
                }))
            }

            const events = result.events
            if (events) {
                const updateLine = events.update_line
                if (updateLine) {
                    const long = updateLine.long
                    chartStore.lastLongLine = {
                        time: Math.floor(new Date(long.time).getTime() / 1000),
                        value: parseFloat(long.value)
                    }

                    const short = updateLine.short
                    chartStore.lastShortLine = {
                        time: Math.floor(new Date(short.time).getTime() / 1000),
                        value: parseFloat(short.value)
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
        
        const lineSeries = chart.addSeries(LineSeries, {
            color: '#2EBD85',
            priceFormat: {
                type: 'percent',
                precision: 100000,
                minMove: 0.0000000001
            },
        })

        const lineSeries2 = chart.addSeries(LineSeries, {
            color: '#F6465D',
            priceScaleId: 'second',
            priceFormat: {
                type: 'percent',
                precision: 100000,
                minMove: 0.0000000001
            },
            autoscaleInfoProvider: () => {
                const range = lineSeries.priceScale().getVisibleRange()
                if (range) {
                    return {
                        priceRange: {
                            minValue: range.from,
                            maxValue: range.to
                        }
                    }
                }
                return null
            }
        })

        setTimeout(() => {
            lineSeries.setData(userStateStore.linesLongHistory)
            lineSeries2.setData(userStateStore.linesShortHistory)
            chart.timeScale().fitContent();
        }, 100)

        if (userStateStore.botWorking) {
            updateInterval = setInterval(() => {
                if (chartStore.lastLongLine && 
                    chartStore.lastLongLine.time != undefined
                ) {
                    lineSeries.update(chartStore.lastLongLine)
                }

                if (chartStore.lastShortLine && 
                    chartStore.lastShortLine.time != undefined
                ) {
                    lineSeries2.update(chartStore.lastShortLine)
                }
            }, 0);
        }

        chart.priceScale('right').applyOptions({
            autoScale: true,
            borderVisible: true,
            borderColor: '#dfdede',
            scaleMargins: {
                top: 0.12,
                bottom: 0.6,
            },
        })

        chart.priceScale('second').applyOptions({
            autoScale: true,
            borderVisible: false,
            borderColor: '#dfdede',
            scaleMargins: {
                top: 0.6,
                bottom: 0.12,
            },
            ticksVisible: false,
        })


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
            chart.resize(
                window.innerWidth - 75,
                window.innerHeight - 17
            )
            chart.timeScale().fitContent()
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
        }
    })
</script>

<template>
    <div class="chart-container">
        <div class="toolbar">
            <div class="chart-exchanges">
                <div>
                    <div>
                        <span class="exchange-name">{{ userStateStore.longExchange }}</span>
                        <img class="exchange-icon" :src="orderBookStore.longExchangeLogo">
                    </div>
                    <span class="longPrice">{{ orderBookStore.longLastPrice }}</span>
                </div>
                <div>
                    <div>
                        <span class="exchange-name">{{ userStateStore.shortExchange }}</span>
                        <img class="exchange-icon" :src="orderBookStore.shortExchangeLogo">
                    </div>
                    <span class="shortPrice">{{ orderBookStore.shortLastPrice }}</span>
                </div>
            </div>
        </div>
        <div class="left-menu">
            <img class="item" src="../assets/icons/exchange.svg">
        </div>
        <div class="chart" ref="container" id="chart"></div>
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

    .exchange-name {
        text-transform: capitalize;
    }
    
    .exchange-icon {
        width: var(--default-icon-size);
        height: var(--default-icon-size);
        display: inline;
        position: sticky;
        top: 10px;
        margin-left: 10px;
    }

    .longPrice, .shortPrice {
        font-weight: 600;
    }

    .longPrice {
        color: var(--color-success);
    }

    .shortPrice {
        color: var(--color-error);
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