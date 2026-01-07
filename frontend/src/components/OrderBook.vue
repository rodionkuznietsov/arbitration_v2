<script setup>
    import { ref, defineExpose } from 'vue';
    let isVisible = ref("display: none;")

    function show() {
        isVisible.value = "display: block;"
    }

    let snapshot_ask = ref([])
    let websoket = null

    let exchanges_list = ref([])

    function exchanges(longExchange, shortExchange) {
        exchanges_list.value = [longExchange, shortExchange]
    }

    function start() {
        if (websoket) return

        websoket = new WebSocket("ws://127.0.0.1:9000")

        websoket.onmessage = (event) => {
            const data = JSON.parse(event.data)

            snapshot_ask.value = data.snapshot.a.map(x => ({
                price: x[0],
                volume: x[1]
            }))
        }

        websoket.onopen = () => {
            websoket.send(JSON.stringify({
                exchange: "bybit".toLowerCase(),
                symbol: "btc"
            }))
        }

        websoket.onclose = () => {
            websoket = null
        }
    }

    function stop() {
        if (websoket) {
            websoket.close()
            websoket = null
        }
        snapshot_ask.value = []
        isVisible.value = "display: none;"
    }

    defineExpose({ show, start, stop, exchanges })
</script>

<template>
    <div id="stakan">
        <div id="order_book" :style="isVisible" v-for="exchange_name in exchanges_list" :key="exchange_name">
            <div id="exchange_name">{{ exchange_name }}</div>
            <div id="order_book_element">
                <table class="orderbook_table">
                    <tr>
                        <th>Цена</th>
                        <th>Обьем</th>
                    </tr>
                    <tr v-for="(row, i) in snapshot_ask.splice(-6)" :key="i">
                        <td>{{ row.price }}</td>
                        <td>{{ row.volume }}</td>
                    </tr>
                </table>
            </div>
        </div>
    </div>
</template>

<style scoped>
    #stakan {
        display: flex;
        gap: 20px;
        margin-top: 20px;
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
        margin-top: 20px;
        text-align: center;
        text-transform: capitalize;
        color: #121212;
        font-weight: 600;
        font-size: 16px;
    }

    .orderbook_table {
        width: 100%;
        text-transform: capitalize;
        text-align: end;
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
        margin-top: 20px;
        margin-bottom: 20px;
    }

    #separator {
        margin-bottom: 30px;
    }
</style>