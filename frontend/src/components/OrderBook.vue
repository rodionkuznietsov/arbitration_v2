<script setup>
    import { useOrderBookStore } from '@/stores/orderbook';
    import { useUserState } from '@/stores/user_state';
    // import { useWebsocketStore } from '@/stores/websocket';
    import { defineExpose, computed } from 'vue';
    import { volumeFormatter, formatCurrency } from '@/utils/formatters';
    import { useAuthStore } from '@/stores/auth';
    import { logBotEvent } from '@/utils/logFetch';

    const isVisible = computed(() => {
        return userState.isBotRunning ? 'display: block;' : "display: none;"
    })
    
    const loading = computed(() => {
        return userState.isBotRunning ? 'display: none;' : "display: block;"
    })

    const authStore = useAuthStore()
    const userState = useUserState()

    const orderBookStore = useOrderBookStore()
    // const ws = useWebsocketStore()

    // let unsubscribe

    function start() {
        const data = userState.get_data()
        // unsubscribe = ws.subscribe(data.symbol.toString(), 'order_book', data.longExchange.toString(), data.shortExchange.toString(), (result) => {            
        //     orderBookStore.updateHeader(
        //         data.symbol,
        //         data.longExchange, 
        //         data.longOrderType,
        //         data.shortExchange,
        //         data.shortOrderType
        //     )

        //     orderBookStore.updateData(result.data.order_book)
        // })

        // Сохраняем лог о старте бота в базу данных
        logBotEvent(
            "bot_start", {
                symbol: data.symbol,
                longExchange: data.longExchange,
                longOrderType: data.longOrderType.toLowerCase(),
                shortExchange: data.shortExchange,
                shortOrderType: data.shortOrderType.toLowerCase(),
            },
            authStore.token
        )
    }

    function stop() {
        const data = userState.get_data()
        // userState.clearValues()
        // orderBookStore.clearValues()
        // if (unsubscribe) {
        //     unsubscribe()

        logBotEvent(
            "bot_stop", {
                symbol: data.symbol,
                longExchange: data.longExchange,
                shortExchange: data.shortExchange
            },
            authStore.token
        )
        // }
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

    defineExpose({ start, stop })
</script>

<template>
    <div class="order_book_title">Книги ордеров</div>
    <div id="stakan">
        <div class="loading_div" :style="loading">
            <img class="loading" src="../assets/img/loading_books.png" draggable="false">
        </div>
        <div id="order_book" :style="isVisible">
            <div id="exchange_name">
                <div class="with_img">
                    <img :src="orderBookStore.longExchangeLogo" alt="">
                    <span>{{ orderBookStore.longExchange }}</span>
                    <span class="ticker_name">{{ orderBookStore.symbol }} - {{ orderBookStore.longOrderType }}</span>
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
                            <span class="volume-value">{{ volumeFormatter(ask.price *  ask.volume) }}</span>
                            <div class="bar" :style="{ 'width': getFillPercentAsk(ask, 'long') + '%'}" style="background-color: var(--color-error-opacity-0_15); right: 0%; top: 0px;"></div>
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
                            <span class="volume-value">{{ volumeFormatter(bid.price *  bid.volume) }}</span>
                            <div class="bar" :style="{ 'width': getFillPercentBid(bid, 'long') + '%'}" style="background-color: var(--color-success-opacity-0_15); right: 0%; top: 0px;"></div>
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
                    <span class="ticker_name">{{ orderBookStore.symbol }} - {{ orderBookStore.shortOrderType }}</span>
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
                            <span class="volume-value">{{ volumeFormatter(ask.price *  ask.volume) }}</span>
                            <div class="bar" :style="{ 'width': getFillPercentAsk(ask, 'short') + '%'}" style="background-color: var(--color-error-opacity-0_15); right: 0%; top: 0px;"></div>
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
                            <span class="volume-value">{{ volumeFormatter(bid.price *  bid.volume) }}</span>
                            <div class="bar" :style="{ 'width': getFillPercentBid(bid, 'short') + '%'}" style="background-color: var(--color-success-opacity-0_15); right: 0%; top: 0px;"></div>
                        </td>
                    </tr>
                </table>
            </div>
        </div>
    </div>
</template>

<style scoped>
    .loading_div {
        position: absolute;
        box-sizing: border-box;
    }

    .loading_div img {
        width: 100%;
        height: 100%;
        object-fit: cover;
        pointer-events: none;
        user-select: none;
        margin-bottom: var(--default-margin-bottom);
    }

    .loading {
        width: 250px;
        height: 250px;
        object-fit: contain;
        background: transparent;
        background-color: transparent;
        pointer-events: none;
        -webkit-user-select: none;
        -moz-user-select: none;
        user-select: none;
        -webkit-user-drag: none;
        -webkit-tap-highlight-color: transparent;
        box-sizing: content-box;
    }

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
        width: 0%;
        bottom: 0;
        right: 0;
        z-index: -1;
        transition: width 0.95s ease-out;
    }

    .ask_row td, .bid_row td{
        position: relative;
    }

    #stakan {
        display: flex;
        gap: 10px;
        margin-top: 10px;
        margin-bottom: var(--default-margin-bottom);
    }

    #order_book {
        width: 100%;
        box-sizing: border-box;
        position: relative;
        border-radius: var(--default-border-radius);
        background-color: var(--default-orderbook-bg);
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
        color: var(--default-font-color);
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
        align-items: center;
    }
</style>