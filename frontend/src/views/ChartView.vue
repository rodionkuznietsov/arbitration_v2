<script setup>
    import { CandlestickSeries, createChart } from 'lightweight-charts';
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

        chart.timeScale().fitContent()
        chart.timeScale().scrollToPosition(0.5, false)
        chart.timeScale().applyOptions({
            fixLeftEdge: true,
            lockVisibleTimeRangeOnResize: true,
            rightOffset: 0,
            borderVisible: false,
        })
    })
    
</script>

<template>
    <div class="chart" ref="container" id="chart"></div>
</template>

<style scoped>
    .chart {
        width: 100%;
        height: 87vh;
    }
</style>