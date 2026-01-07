<script setup>
    import { ref, defineExpose } from 'vue';
    let isVisible = ref("display: none;")

    function show() {
        isVisible.value = "display: block;"
    }

    let snapshot_ask = ref([])

    let websoket = null
    const exchanges = ["Bybit", "Mexc"]

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

        websoket.onclose = () => {
            websoket = null
        }
    }

    function stop() {
        if (websoket) {
            websoket.close()
            websoket = null

            snapshot_ask.value = []
            isVisible.value = "display: none;"
        }
    }

    defineExpose({ show, start, stop })
</script>

<template>
    <div id="order_book" :style="isVisible" v-for="exchange_name in exchanges" :key="exchange_name">
        <div id="order_book_element">
            <div id="exchange_name">{{ exchange_name }}</div>
            <div id="order_book_labels">
                <div>
                    <span>Цена</span>
                    <div id="sell_label" v-for="(row, i) in snapshot_ask.splice(-6)" :key="i">
                        {{ row.price }}
                    </div>
                </div>
                <div>
                    <span>Обьем $</span>
                    <div id="sell_label" v-for="(row, i) in snapshot_ask.splice(-6)" :key="i">
                        {{ row.volume }}
                    </div>
                </div>
            </div> 
        </div>
    </div>
</template>

<style scoped>
    #order_book {
        margin-top: 20px;
        display: flex;
        gap: 20px;
        position: relative;
        border-radius: 8px;
        background-color: rgba(255, 255, 255, 0.7);
        flex-direction: row;
    }

    #order_book_element {
        width: 100%;
        padding: 8px;
        border: 1px solid transparent;
        color: #121212;
        font-weight: 600;
        border-radius: 8px;
        font-size: 16px;
        margin-top: 10px;
        backdrop-filter: blur(8px); 
        -webkit-backdrop-filter: blur(8px);
        outline: none;
        box-sizing: border-box;
        cursor: pointer;
        text-transform: uppercase;
        text-align: end;
        overflow-y: auto;
    }

    #exchange_name {
        text-align: center;
        text-transform: capitalize;
    }

    #order_book_labels {
        display: flex;
        justify-content: space-evenly;
        gap: 20px;
        text-transform: capitalize;
    }

    .order_book_prices {
        margin-top: 10px;
    }

    .buy_label, #sell_label {
        margin-top: 10px;
        display: flex;
        flex-direction: column;
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