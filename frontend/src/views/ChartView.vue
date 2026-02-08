<script setup>
    import { CandlestickSeries, createChart, CrosshairMode } from 'lightweight-charts';
import { onMounted, ref } from 'vue';

    const container = ref(null)

    onMounted(() => {
        const chartOptions = {
            width: container.value.clientWidth,
            height: container.value.clientHeight,

            timeScale: {
                timeVisible: true,
                secondsVisible: false,
            },
                    
            layout: {
                background: { color: '#202630', type: 'solid' },
                textColor: '#EAECEF',
            },
            crosshair: {
                mode: CrosshairMode.Normal,
            },
            grid: {
                vertLines: {
                    color: '#333B47',
                },
                horzLines: {
                    color: '#333B47',
                },
            },
        }
        
        const chart = createChart(container.value, chartOptions);
        const candleSeries = chart.addSeries(CandlestickSeries,{
            upColor: '#2EBD85',
            downColor: '#F6465D',
            borderDownColor: '#F6465D',
            borderUpColor: '#2EBD85',
            wickDownColor: '#F6465D',
            wickUpColor: '#2EBD85',
        });
        candleSeries.setData([
            { time: Math.floor(new Date('2026-01-07T00:00:00Z').getTime() / 1000), open: 0.0, high: 0.19, low: 0.09, close: 0.05 },
            { time: Math.floor(new Date('2026-01-07T00:05:00Z').getTime() / 1000), open: 0.05, high: 0.15, low: 0.10, close: 0.09 },
            { time: Math.floor(new Date('2026-01-07T00:10:00Z').getTime() / 1000), open: 0.09, high: 0.18, low: 0.13, close: 0.17 },
            { time: Math.floor(new Date('2026-01-07T00:15:00Z').getTime() / 1000), open: 0.17, high: 0.22, low: 0.16, close: 0.2 },
            { time: Math.floor(new Date('2026-01-07T00:20:00Z').getTime() / 1000), open: 0.2, high: 0.25, low: 0.18, close: 0.07 },
        ]);

        chart.priceScale('right').applyOptions({
            borderVisible: false,
            scaleMargins: {
                top: 0.4,
                bottom: 0.4,
            },
        })

        const vr = chart.timeScale().getVisibleLogicalRange();
        chart.timeScale().setVisibleLogicalRange({
            from: vr.from - 5 * 60, // 5 minutes before
            to: vr.to + 5 * 60,
        });

        chart.timeScale().applyOptions({
            lockVisibleTimeRangeOnResize: false,
            borderVisible: true,
            barSpacing: 20,
            rightOffset: 5,
            tickMarkFormatter: (time, tickMarkType, locale) => {
                const date = new Date(time * 1000);
                return date.toLocaleTimeString(locale, { hour: '2-digit', minute: '2-digit' });
            }
        })
    })
    
</script>

<template>
    <div class="toolbar">
        <input class="toolbar-button" type="button" value="5m">
    </div>
    <div class="chart" ref="container" id="chart"></div>
    <div class="title_bg">Arbitration Bot</div>
</template>

<style scoped>
    .chart {
        width: 100%;
        height: 87vh;
    }

    .toolbar {
        display: flex;
        justify-content: flex-start;
        padding: 10px;
        position: absolute;
        z-index: 1000000000;
    }

    .toolbar-button {
        background-color: var(--default-input-color);
        color: #EAECEF;
        border: none;
        padding: 8px 16px;
        margin-right: 10px;
        cursor: pointer;
        border-radius: 4px;
        font-size: var(--default-font-size);
    }

    .title_bg {
        position: absolute;
        top: 50%;
        left: 50%;
        transform: translate(-50%, -50%);
        font-size: 18px;
        color: #ffffff27;
        pointer-events: none;
        z-index: 1000000000;
        font-weight: bold;
    }
</style>