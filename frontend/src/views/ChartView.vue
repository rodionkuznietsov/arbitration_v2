<script setup>
    import { useChartStore } from '@/stores/chart';
    // import { useOrderBookStore } from '@/stores/orderbook';
    import { useUserState } from '@/stores/user_state';
    import { useWebsocketStore } from '@/stores/websocket';
    import { createChart, CrosshairMode } from 'lightweight-charts';
    import { computed, onActivated, onDeactivated, ref, watch } from 'vue';
    import { volumeFormatter } from '@/utils/formatters';
    import ChartLeftMenu from '@/components/chart/ChartLeftMenu.vue';
    import ChartHeader from '@/components/chart/ChartHeader.vue';

    const userStateStore = useUserState()
    const ws = useWebsocketStore()
    let unsubscribe

    const container = ref(null)
    const chartStore = useChartStore()

    const longVolume = computed(() => {
        return chartStore.swapActive ? chartStore.shortVolume24hr : chartStore.longVolume24hr
    })

    const shortVolume = computed(() => {
        return chartStore.swapActive ? chartStore.longVolume24hr : chartStore.shortVolume24hr
    })

    let stopVolumeWatch

    onActivated(() => {
        unsubscribe = ws.subscribe(userStateStore.ticker, 'chart', userStateStore.longExchange, userStateStore.shortExchange, (result) => {                                                            

            const tempLongHistory = result.data.lines_history ? result.data.lines_history.long.map(line => ({
                time: Math.floor(new Date(line.time).getTime()),
                value: parseFloat(line.value)
            })) : {}

            const tempShortHistory = result.data.lines_history ? result.data.lines_history.short.map(line => ({
                time: Math.floor(new Date(line.time).getTime()),
                value: parseFloat(line.value)
            })) : {}

            chartStore.linesLongHistory = tempLongHistory
            chartStore.linesShortHistory = tempShortHistory 

            const update_long_value = result.data.update_line ? {
                time: Math.floor(new Date(result.data.update_line.long.time).getTime()),
                value: parseFloat(result.data.update_line.long.value)
            } : {}

            const update_short_value = result.data.update_line ? {
                time: Math.floor(new Date(result.data.update_line.short.time).getTime()),
                value: parseFloat(result.data.update_line.short.value)
            } : {}
            
            chartStore.lastLongLine = update_long_value
            chartStore.lastShortLine = update_short_value

            if (result.data.volume24h) {
                chartStore.longVolume24hr = volumeFormatter(result.data.volume24h.long.value)
                chartStore.shortVolume24hr = volumeFormatter(result.data.volume24h.short.value)
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
        
        chartStore.chart = createChart(container.value, chartOptions);
        
        chartStore.legend = document.createElement('div')
        chartStore.legend.style = `
            position: absolute;
            left: 56px;
            top: 70px;
            z-index: 10000;
            font-size: 14px;
            color: rgb(31, 31, 31);
            user-select: none;
            pointer-events: none;
        `
        chartStore.legend.style.color = '#1f1f1f'
        chartStore.legend.style.userSelect = 'none'
        chartStore.legend.style.pointerEvents = 'none'
        chartStore.legend.innerHTML = `<div><span>Оборот за 24 часа:</span><div style="text-transform: capitalize;">` + chartStore.longExchange + `: `+ longVolume.value +`</div><div style="text-transform: capitalize;">`+ chartStore.shortExchange +`: `+ shortVolume.value +`</div></div>`
        container.value.appendChild(chartStore.legend)
        
        stopVolumeWatch = watch(
            () => [chartStore.longVolume24hr, chartStore.shortVolume24hr],
            () => {
                chartStore.legend.innerHTML = `<div><span>Оборот за 24 часа:</span><div style="text-transform: capitalize;">` + chartStore.longExchange + `: `+ longVolume.value +`</div><div style="text-transform: capitalize;">`+ chartStore.shortExchange +`: `+ shortVolume.value +`</div></div>`
            }
        )
        chartStore.createSeries()

        chartStore.chart.timeScale().applyOptions({
            lockVisibleTimeRangeOnResize: false,
            borderColor: '#dfdede',
            borderVisible: true,
            tickMarkFormatter: (time, tickMarkType, locale) => {
                const date = new Date(time * 1000);
                return date.toLocaleTimeString(locale, { hour: '2-digit', minute: '2-digit' });
            },
        })

        window.addEventListener('resize', () => {
            if (chartStore.chart) {
                chartStore.chart.resize(
                    window.innerWidth - 75,
                    window.innerHeight - 17
                )
                chartStore.chart.timeScale().fitContent()
            }
        })
    })

    onDeactivated(() => {
        if (chartStore.scope) {
            chartStore.scope.stop()
        }

        if (stopVolumeWatch) {
            stopVolumeWatch()
        }

        if (chartStore.finished) {
            if (unsubscribe) {
                unsubscribe()
                unsubscribe = null
            }
        } 

        if (chartStore.chart) {
            chartStore.chart.remove();
            chartStore.chart = null;
        }

        if (chartStore.legend) {
            chartStore.legend.remove()
            chartStore.legend = null
        }

        window.removeEventListener('resize', () => {})
    })
</script>

<template>
    <div class="page">
        <div class="scroll">
            <div class="chart-container">
                <ChartLeftMenu class="chart-container-item"/>
                <ChartHeader class="chart-container-item" />
                <div class="chart chart-container-item" ref="container" id="chart">
                    <div class="ticker">{{ userStateStore.ticker }}</div>
                    <img class="chart-background" src="../assets/img/chart-background.png" alt="">
                </div>
            </div>
        </div>
    </div>
</template>

<style scoped>
    .chart-container {
        display: grid;
        grid-template-columns: auto 1fr;
        grid-template-rows: auto 4fr 1fr;
        top: 0;
        position: absolute;
        left: 0;
        width: 100%;
        height: 100vh;
        align-content: start;
        justify-items: start;
    }

    .chart-container .chart-container-item:nth-child(1) {
        grid-column: 1;
        grid-row: 1 / -1;
    }

    .chart-container .chart-container-item:nth-child(2) {
        grid-column: 2;
        grid-row: 1;
    }

    .chart-container .chart-container-item:nth-child(3) {
        grid-column: 2;
        grid-row: 2;
    }

    .chart {
        width: 100%;
    }

    .chart-background {
        position: absolute;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;

        object-fit: cover;
        opacity: 0.1;

        pointer-events: none; /* чтобы не мешала графику */
        z-index: 2;
        filter: blur(2px);
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

</style>