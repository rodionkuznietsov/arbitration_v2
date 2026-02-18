<script setup>
    import { useChartStore } from '@/stores/chart';
    import { useUserState } from '@/stores/user_state';
    import { useWebsocketStore } from '@/stores/websocket';
    import { CandlestickSeries, createChart, CrosshairMode } from 'lightweight-charts';
    import { onActivated, onDeactivated, ref } from 'vue';

    const userStateStore = useUserState()
    
    const ws = useWebsocketStore()
    let unsubscribe

    let chart;
    const container = ref(null)

    const chartStore = useChartStore()
    let updateInterval

    onActivated(() => {
        unsubscribe = ws.subscribe(userStateStore.ticker, 'candles_history', userStateStore.longExchange, userStateStore.shortExchange, (msg) => {
            userStateStore.candles_history = msg.candles.map(c => ({
                time: Math.floor(new Date(c.timestamp).getTime() / 1000),
                ticker: c.symbol,
                open: parseFloat(c.open.toString()),
                high: parseFloat(c.high.toString()),
                low: parseFloat(c.low.toString()),
                close: parseFloat(c.close.toString()),
            }))

            const candle_event = msg.events?.candle

            if (candle_event) {
                chartStore.lastCandle = {
                    time: Math.floor(new Date(candle_event.timestamp).getTime() / 1000),
                    ticker: candle_event.symbol,
                    open: parseFloat(candle_event.open.toString()),
                    high: parseFloat(candle_event.high.toString()),
                    low: parseFloat(candle_event.low.toString()),
                    close: parseFloat(candle_event.close.toString()),
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
        const candleSeries = chart.addSeries(CandlestickSeries,{
            upColor: '#2EBD85',
            downColor: '#F6465D',
            borderDownColor: '#F6465D',
            borderUpColor: '#2EBD85',
            wickDownColor: '#F6465D',
            wickUpColor: '#2EBD85',
        });

        setTimeout(() => {
            candleSeries.setData(userStateStore.candles_history)
            chart.timeScale().fitContent();
        }, 100)

        if (userStateStore.botWorking) {
            updateInterval = setInterval(() => {
                if (chartStore.lastCandle) {
                    candleSeries.update(chartStore.lastCandle)
                }
            }, 0);
        }

        chart.priceScale('right').applyOptions({
            borderVisible: true,
            scaleMargins: {
                top: 0.4,
                bottom: 0.4,
            },
        })

        chart.timeScale().applyOptions({
            lockVisibleTimeRangeOnResize: false,
            borderVisible: true,
            tickMarkFormatter: (time, tickMarkType, locale) => {
                const date = new Date(time * 1000);
                return date.toLocaleTimeString(locale, { hour: '2-digit', minute: '2-digit' });
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
        }
    })
</script>

<template>
    <div>
        <div class="toolbar">
            <div class="intervals">
                <input class="toolbar-button" type="button" value="5m">
            </div>
            <input type="button" value="Вход" class="toolbar-button">
            <input type="button" value="Выход" class="toolbar-button">
        </div>
        <div class="chart" ref="container" id="chart"></div>
        <div class="title_bg">Arbitration Bot</div>
    </div>
</template>

<style scoped>
    .chart {
        width: 100%;
        height: 88vh;
    }

    .toolbar {
        display: flex;
        justify-content: flex-start;
        padding: var(--default-padding);
        position: absolute;
        z-index: 1000000000;
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