<script setup>
import { ref, defineExpose, defineProps, defineEmits } from 'vue';
    const props = defineProps({
        modelValue: Boolean
    })

    const isVisible = ref("display: none;")
    const emit = defineEmits(["update:modelValue"])
    const isWarningStatus = ref(props.modelValue)

    let websoket = null
    const formData = ref({})

    const longAsks = ref([])
    const longBids = ref([])
    const longLastPrice = ref(0.0)
    const longExchange = ref('Binance')

    const shortAsks = ref([])
    const shortBids = ref([])
    const shortLastPrice = ref(0.0)
    const shortExchange = ref('Bybit')

    function exchanges(
        data
    ) {
        formData.value = data
        longExchange.value = formData.value.exchanges.longExchange
        shortExchange.value = formData.value.exchanges.shortExchange
    }

    function start() {
        if (websoket) return

        websoket = new WebSocket("ws://localhost:9000")

        websoket.onerror = () => {
            console.log("[Websocket] Прервано соединение")
            emit("update:modelValue", true)
        }

        websoket.onmessage = (event) => {
            const data = JSON.parse(event.data)

            longAsks.value = data.book1.snapshot.a.map(x => ({
                price: x[0],
                volume: x[1],
            }))

            longBids.value = data.book1.snapshot.b.map(x => ({
                price: x[0],
                volume: x[1],
            }))

            longLastPrice.value = data.book1.snapshot.last_price

            shortAsks.value = data.book2.snapshot.a.map(x => ({
                price: x[0],
                volume: x[1],
            }))

            shortBids.value = data.book2.snapshot.b.map(x => ({
                price: x[0],
                volume: x[1],
            }))

            shortLastPrice.value = data.book2.snapshot.last_price
        }

        websoket.onopen = () => {
            websoket.send(JSON.stringify(formData.value))
        }

        websoket.onclose = () => {
            websoket = null
        }
    }

    function show() {
        isVisible.value = "display: block;"
    }

    function stop() {
        if (websoket) {
            websoket.close()
            websoket = null
        }
        isVisible.value = "display: none;"
    }

    function formatCurrency(value) {
        if (typeof value != 'number') {
            return value;
        }

        return new Intl.NumberFormat(
            "en-US", {
                minimumFractionDigits: 2,
                maximumFractionDigits: 2,
            }
        ).format(value)
    }

    defineExpose({ show, start, stop, exchanges, isWarningStatus })
</script>

<template>
    <div id="stakan">
        <div id="order_book" :style="isVisible">
            <div id="exchange_name">{{ longExchange }}</div>
            <div id="order_book_element">
                <table class="orderbook_table">
                    <tr>
                        <th>Цена</th>
                        <th>Обьем $</th>
                    </tr>
                    <tr v-for="ask in longAsks.splice(-6)" :key="ask">
                        <td class="sell_label"> {{ formatCurrency(ask.price) }} </td>
                        <td class="sell_label"> {{ formatCurrency(ask.price *  ask.volume) }} </td>
                    </tr>

                    <tr>
                        <td colspan="2" class="center_label" tabindex="3">
                            {{ formatCurrency(longLastPrice) }}
                        </td>
                    </tr>

                    <tr v-for="bid in longBids.splice(0, 6)" :key="bid">
                        <td class="buy_label"> {{ formatCurrency(bid.price) }} </td>
                        <td class="buy_label"> {{ formatCurrency(bid.price *  bid.volume) }} </td>
                    </tr>
                </table>
            </div>
        </div>

        <div id="order_book" :style="isVisible">
            <div id="exchange_name">{{ shortExchange }}</div>
            <div id="order_book_element">
                <table class="orderbook_table">
                    <tr>
                        <th>Цена</th>
                        <th>Обьем $</th>
                    </tr>
                    <tr v-for="ask in shortAsks.splice(-6)" :key="ask">
                        <td class="sell_label"> {{ formatCurrency(ask.price) }} </td>
                        <td class="sell_label"> {{ formatCurrency(ask.price *  ask.volume) }} </td>
                    </tr>

                    <tr>
                        <td colspan="2" class="center_label" tabindex="3">
                            {{ formatCurrency(shortLastPrice) }}
                        </td>
                    </tr>

                    <tr v-for="bid in shortBids.splice(0, 6)" :key="bid">
                        <td class="buy_label"> {{ formatCurrency(bid.price) }} </td>
                        <td class="buy_label"> {{ formatCurrency(bid.price *  bid.volume) }} </td>
                    </tr>
                </table>
            </div>
        </div>
    </div>
</template>

<style scoped>
    #stakan {
        display: flex;
        gap: 10px;
        margin-top: 10px;
    }

    #order_book {
        width: 100%;
        box-sizing: border-box;
        position: relative;
        border-radius: 8px;
        background-color: rgba(255, 255, 255, 0.7);
    }

    #order_book_element {
        width: 100%;
        padding: 8px;
        border: 1px solid transparent;
        color: #121212;
        font-weight: 600;
        border-radius: 8px;
        font-size: 16px;
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
        color: #121212;
        font-weight: 600;
        font-size: 16px;
    }

    .orderbook_table {
        width: 100%;
        text-transform: capitalize;
        text-align: center;
    }

    .order_book_prices {
        margin-top: 10px;
    }

    .buy_label, #sell_label {
        margin-top: 10px;
    }

    .buy_label {
        color: green;
        /* box-shadow: 1px 1px 80px green; */
    }

    .sell_label {
        color: rgb(151, 15, 15);
        /* box-shadow: 1px 1px 80px rgb(151, 15, 15); */
    }

    #mid_price {
        margin-top: 10px;
        margin-bottom: 10px;
    }

    #separator {
        margin-bottom: 30px;
    }

    .center_label {
        text-align: center;
        color: rgb(151, 15, 15);
        border-top: 1px dashed #121212;
        border-bottom: 1px dashed #121212;
    }
</style>