<script setup>
    import { useOrderBookStore } from '@/stores/orderbook';
    import { useUserState } from '@/stores/user_state';
    import { useWebsocketStore } from '@/stores/websocket';
    import { ref, defineExpose, defineProps, defineEmits } from 'vue';

    const props = defineProps({
        modelValue: Boolean
    })

    const isVisible = ref("display: none;")
    const emit = defineEmits(["update:modelValue"])
    const isWarningStatus = ref(props.modelValue)

    const userState = useUserState()
    const orderBookStore = useOrderBookStore()
    const ws = useWebsocketStore()
    let unsubscribe

    function start() {
        emit("update:modelValue", true)
        const data = userState.get_data()

        unsubscribe = ws.subscribe(data.ticker.toString(), 'order_book', data.longExchange.toString(), data.shortExchange.toString(), (msg) => {
            orderBookStore.updateHeader(
                data.ticker,
                data.longExchange, 
                data.longOrderType,
                data.shortExchange,
                data.shortOrderType
            )

            orderBookStore.updateData(msg.books)
            isVisible.value = "display: block;"
        })
    }

    function stop() {
        isVisible.value = "display: none;"
        userState.clearValues()
        orderBookStore.clearValues()
        unsubscribe?.()
    }

    function formatCurrency(value) {
        if (typeof value != 'number') {
            return value;
        }

        return new Intl.NumberFormat(
            "en-US", {
                minimumFractionDigits: 2,
                maximumFractionDigits: 10,
            }
        ).format(value)
    }

    function formatVolume(value) {
        if (typeof value != 'number') {
            return value;
        }

        return new Intl.NumberFormat(
            "en-US", {
                minimumFractionDigits: 1,
                maximumFractionDigits: 2,
            }
        ).format(value)
    }   

    function getFillPercentAsk(value, method) {
        const source = method == 'long' ? orderBookStore.longAsks : orderBookStore.shortAsks
        const max = Math.max(...source.map(a => a.price * a.volume))
        return (value.price * value.volume) / max * 100
    }

    function getFillPercentBid(value, method) {
        const source = method == 'long' ? orderBookStore.longBids : orderBookStore.shortBids
        const max = Math.max(...source.map(b => b.price * b.volume))
        return (value.price * value.volume) / max * 100
    }

    defineExpose({ start, stop, isWarningStatus })
</script>

<template>
    <div class="order_book_title">Книги ордеров</div>
    <div id="stakan">
        <div id="order_book" :style="isVisible">
            <div id="exchange_name">
                <div class="with_img">
                    <img :src="orderBookStore.longExchangeLogo" alt="">
                    <span>{{ orderBookStore.longExchange }}</span>
                    <span class="ticker_name">{{ orderBookStore.ticker }} - {{ orderBookStore.longOrderType }}</span>
                </div>
            </div>
            <div id="order_book_element">
                <table class="orderbook_table">
                    <tr>
                        <th>Цена</th>
                        <th>Обьем $</th>
                    </tr>
                    <tr v-for="ask in orderBookStore.longAsks" :key="ask.price" class="ask_row">
                        <td class="sell_label"> {{ formatCurrency(ask.price) }} </td>
                        <td class="sell_label volume-bar">
                            <span class="volume-value">{{ formatVolume(ask.price *  ask.volume) }}</span>
                            <div class="bar" :style="{ 'min-width': getFillPercentAsk(ask, 'long') + '%'}" style="width: 0%; background-color: var(--color-error-opacity-0_15); right: 0px; top: 0px;"></div>
                        </td>
                    </tr>
                    <tr>
                        <td colspan="2" class="mid_price" tabindex="3">
                            <div class="with_arrow" :class="orderBookStore.longArrow == '⬇' ? 'down' : 'up'">
                                <div>{{ orderBookStore.longArrow }} </div>
                                <div>{{ formatCurrency(orderBookStore.longLastPrice) }}</div>
                            </div>
                        </td>
                    </tr>

                    <tr v-for="bid in orderBookStore.longBids" :key="bid.price" class="bid_row">
                        <td class="buy_label"> {{ formatCurrency(bid.price) }} </td>
                        <td class="buy_label volume-bar">
                            <span class="volume-value">{{ formatVolume(bid.price *  bid.volume) }}</span>
                            <div class="bar" :style="{ 'min-width': getFillPercentBid(bid, 'long') + '%'}" style="width: 0%; background-color: var(--color-success-opacity-0_15); right: 0px; top: 0px;"></div>
                        </td>
                    </tr>
                </table>
            </div>
        </div>

        <div id="order_book" :style="isVisible">
            <div id="exchange_name">
                <div class="with_img">
                    <img :src="orderBookStore.shortExchangeLogo" alt="">
                    <span>{{ orderBookStore.shortExchange }}</span>
                    <span class="ticker_name">{{ orderBookStore.ticker }} - {{ orderBookStore.shortOrderType }}</span>
                </div>
            </div>
            <div id="order_book_element">
                <table class="orderbook_table">
                    <tr>
                        <th>Цена</th>
                        <th>Обьем $</th>
                    </tr>
                    <tr v-for="ask in orderBookStore.shortAsks" :key="ask.price" class="ask_row">
                        <td class="sell_label"> {{ formatCurrency(ask.price) }} </td>
                        <td class="sell_label volume-bar">
                            <span class="volume-value">{{ formatVolume(ask.price *  ask.volume) }}</span>
                            <div class="bar" :style="{ 'min-width': getFillPercentAsk(ask, 'short') + '%'}" style="width: 0%; background-color: var(--color-error-opacity-0_15); right: 0px; top: 0px;"></div>
                        </td>
                    </tr>
                    <tr>
                        <td colspan="2" class="mid_price" tabindex="3">
                            <div class="with_arrow" :class="orderBookStore.shortArrow == '⬇' ? 'down' : 'up'">
                                <div>{{ orderBookStore.shortArrow }} </div>
                                <div>{{ formatCurrency(orderBookStore.shortLastPrice) }}</div>
                            </div>
                        </td>
                    </tr>

                    <tr v-for="bid in orderBookStore.shortBids" :key="bid.price" class="bid_row">
                        <td class="buy_label"> {{ formatCurrency(bid.price) }} </td>
                        <td class="buy_label volume-bar">
                            <span class="volume-value">{{ formatVolume(bid.price *  bid.volume) }}</span>
                            <div class="bar" :style="{ 'min-width': getFillPercentBid(bid, 'short') + '%'}" style="width: 0%; background-color: var(--color-success-opacity-0_15); right: 0px; top: 0px;"></div>
                        </td>
                    </tr>
                </table>
            </div>
        </div>
    </div>
</template>

<style scoped>
    .order_book_title {
        text-align: center;
        margin-top: 10px;
    }

    .ask_row, .bid_row {
        position: relative;
    }

    .bar {
        position: absolute;
        top: 0;
        bottom: 0;
        left: auto;
        z-index: -1;
        transition: width 0.12s linear;
    }

    .ask_row td, .bid_row td{
        position: relative;
    }

    #stakan {
        display: flex;
        gap: 10px;
        margin-top: 10px;
    }

    #order_book {
        width: 100%;
        box-sizing: border-box;
        position: relative;
        border-radius: var(--default-border-radius);
        background-color: #303c5199;
    }

    #order_book_element {
        width: 100%;
        border: 1px solid transparent;
        font-weight: 600;
        border-radius: var(--default-border-radius);
        backdrop-filter: blur(8px); 
        -webkit-backdrop-filter: blur(8px);
        outline: none;
        box-sizing: border-box;
        cursor: pointer;
        text-transform: uppercase;
        overflow-y: auto;
        display: flex;
    }
    
    #exchange_name {
        margin-top: 10px;
        text-align: center;
        text-transform: capitalize;
        font-weight: 600;
    }

    #exchange_name img {
        width: var(--default-icon-size);
        height: var(--default-icon-size);
    }

    .with_img {
        display: flex;
        justify-content: center;
        gap: 10px;
        align-items: center;
    }

    .ticker_name {
        text-transform: uppercase;
        font-size: 10px;
        margin-top: -5px;
        margin-left: -5px;
        color: #d8d8d8;
    }

    .orderbook_table {
        width: 100%;
        text-transform: capitalize;
        text-align: center;
    }

    .order_book_prices {
        margin-top: 10px;
    }

    .buy_label, .sell_label {
        margin-top: 10px;
    }

    .buy_label, .up {
        color: var(--color-success);
    }

    .sell_label, .down {
        color: var(--color-error);
    }

    .mid_price {
        text-align: center;
        border-top: 1px dashed var(--default-font-color);
        border-bottom: 1px dashed var(--default-font-color);
    }

    .with_arrow {
        display: flex;
        justify-content: center;
        gap: 5px;
    }
</style>