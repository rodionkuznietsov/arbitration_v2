<script setup>
    import { useChartStore } from '@/stores/chart';
    import { useOrderBookStore } from '@/stores/orderbook';
    import { computed } from 'vue';

    const chartStore = useChartStore()
    const orderBookStore = useOrderBookStore()

    const lastLongPriceSwapped = computed(() => {
        return chartStore.swapActive ? chartStore.priceFormat(orderBookStore.shortFirstAskPrice) : chartStore.priceFormat(orderBookStore.longFirstBidPrice)
    })

    const lastShortPriceSwapped = computed(() => {
        return chartStore.swapActive ? chartStore.priceFormat(orderBookStore.longFirstBidPrice) : chartStore.priceFormat(orderBookStore.shortFirstAskPrice)
    })
    
</script>

<template>
    <div class="chart-header">
        <div class="long-ex">
            <div>
                <span class="span-text">{{ chartStore.longExchange ? chartStore.longExchange : "unknown" }}</span>
                <img class="exchange-icon" :src="chartStore.longExchangeLogo ? chartStore.longExchangeLogo : ''">
            </div>
            <div class="chart-header-element long-eleement">
                <img class="icon_svg long" src="@/assets/icons/long.svg">
                <span class="price price-write-left">{{ lastLongPriceSwapped ? lastLongPriceSwapped : 0.0 }}</span>
            </div>
        </div>
        <div class="short-ex">
            <div>
                <span class="span-text">{{ chartStore.shortExchange ? chartStore.shortExchange : "unknown" }}</span>
                <img class="exchange-icon" :src="chartStore.shortExchangeLogo ? chartStore.shortExchangeLogo : ''">
            </div>
            <div class="chart-header-element short-element">
                <img class="icon_svg short" src="@/assets/icons/short.svg">
                <span class="price price-write-right">{{ lastShortPriceSwapped ? lastShortPriceSwapped : 0.0 }}</span>
            </div> 
        </div>
    </div>
</template>

<style scoped>
    .chart-header {
        width: 100%;
        background: var(--chart-bg-header-color);
        height: auto;
        display: flex;
        flex-direction: row;
        justify-content: space-between;
        align-items: center;
        border-bottom: 1px solid var(--color-chart-header-border-bottom);
        padding: var(--default-padding);
        box-sizing: border-box;
        z-index: 3;
    }

    .price {
        width: 0;
        display: block;
    }

    .price-write-right {
        margin-right: 5px;
        direction: rtl;
    }

    .price-write-left {
        margin-left: 5px;
        direction: ltr;
    }

    .chart-header-element {
        display: flex;
        flex-direction: row;
        align-items: center;
    }

    .long-element {
        justify-content: flex-start;
    }

    .short-element {
        justify-content: flex-start;
        display: flex;
        flex-direction: row-reverse;
    }
</style>