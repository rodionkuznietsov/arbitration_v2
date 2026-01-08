<script setup>
import { ref, defineExpose, defineProps, defineEmits } from 'vue';
    const props = defineProps({
        modelValue: Boolean
    })

    const isVisible = ref("display: none;")
    const emit = defineEmits(["update:modelValue"])
    const isWarningStatus = ref(props.modelValue)

    let websoket = null
    let snapshot_ask = ref([])
    let snapshot_bid = ref([])
    let snapshot_last_price = ref(0.0)
    let exchanges_list = ref([])

    function exchanges(longExchange, shortExchange) {
        exchanges_list.value = [longExchange, shortExchange]
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

            snapshot_ask.value = data.snapshot.a.map(x => ({
                price: x[0],
                volume: x[1]
            }))

            snapshot_bid.value = data.snapshot.b.map(x => ({
                price: x[0],
                volume: x[1]
            }))

            snapshot_last_price.value = formatCurrency(data.snapshot.last_price)
        }

        websoket.onopen = () => {
            websoket.send(JSON.stringify({
                order_type: "spot".toLowerCase(),
                exchange: exchanges_list.value[0].toLowerCase(),
                symbol: "btc".toLowerCase()
            }))
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
        snapshot_ask.value = []
        isVisible.value = "display: none;"
    }

    function formatCurrency(value) {
        if (typeof value != 'number') {
            return value;
        }

        return new Intl.NumberFormat(
            "en-US"
        ).format(value)
    }

    defineExpose({ show, start, stop, exchanges, isWarningStatus })
</script>

<template>
    <div id="stakan">
        <div id="order_book" :style="isVisible" v-for="exchange_name in exchanges_list" :key="exchange_name">
            <div id="exchange_name">{{ exchange_name }}</div>
            <div id="order_book_element">
                <table class="orderbook_table">
                    <tr>
                        <th>Цена</th>
                        <th>Обьем $</th>
                    </tr>
                    <tr v-for="(row, i) in snapshot_ask.splice(-6)" :key="i">
                        <td class="sell_label">{{ formatCurrency(row.price) }}</td>
                        <td class="sell_label">{{ formatCurrency(row.volume * row.price) }}</td>
                    </tr>
                    <tr>
                        <td colspan="2" class="center_label"> ⬇ {{ snapshot_last_price }}</td>
                    </tr>
                    <tr v-for="(row, i) in snapshot_bid.splice(0, 6)" :key="i">
                        <td class="buy_label">{{ formatCurrency(row.price) }}</td>
                        <td class="buy_label">{{ formatCurrency(row.volume * row.price) }}</td>
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