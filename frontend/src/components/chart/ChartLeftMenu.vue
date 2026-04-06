<script setup>
    import exchangeIconTrue from '@/assets/icons/exchange_true.svg'
    import exchangeIconFalse from '@/assets/icons/exchange_false.svg'
    import { useChartStore } from '@/stores/chart'

    const chartStore = useChartStore()

    function swapExchange() {
        const tempLong = chartStore.longExchange
        const tempShort = chartStore.shortExchange
        const tempLongLogo = chartStore.longExchangeLogo
        const tempShortLogo = chartStore.shortExchangeLogo

        chartStore.longExchange = tempShort
        chartStore.shortExchange = tempLong
        chartStore.longExchangeLogo = tempShortLogo
        chartStore.shortExchangeLogo = tempLongLogo

        chartStore.swapActive.value = !chartStore.swapActive.value
        // chartStore.legend.innerHTML = `<div><span>Оборот за 24 часа:</span><div style="text-transform: capitalize;">` + chartStore.longExchange + `: `+ longVolume.value +`</div><div style="text-transform: capitalize;">`+ chartStore.shortExchange +`: `+ shortVolume.value +`</div></div>`
        chartStore.createSeries()

        changeLineSeriesColor()
    }

    function changeLineSeriesColor() {
        console.log()
        // swapActive.value ? inSpreadSeries.applyOptions({
        //     color: '#F6465D'
        // }) : inSpreadSeries.applyOptions({
        //     color: '#2EBD85'
        // })

        // !swapActive.value ? outSpreadSeries.applyOptions({
        //     color: '#F6465D'
        // }) : outSpreadSeries.applyOptions({
        //     color: '#2EBD85'
        // })
    }
</script>

<template>
    <div class="left-menu">
        <img @click="swapExchange()" class="icon_svg icon_svg_click" :src="
        isHovered ? exchangeIconTrue : exchangeIconFalse"
        @mouseenter="isHovered = true"
        @mouseleave="isHovered = false"
        title="Поменять направления"
        />
    </div>
</template>

<style scoped>
    .left-menu {
        background: var(--chart-bg-left-menu-color);
        padding: var(--chart-padding-left-menu-px);
        z-index: 3;
        border-right: 1px solid var(--color-chart-border-left-menu-right);
    }

    .icon_svg_click {
        background-color: var(--default-input-bg);
        padding: var(--default-padding);
        border-radius: var(--default-border-radius);
        pointer-events: all;
    }
</style>