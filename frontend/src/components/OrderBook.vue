<script setup>
import { ref } from 'vue';

    let websoket = new WebSocket("ws://127.0.0.1:9000");

    let ask_data = ref([])
    let bid_data = ref([])

    websoket.onmessage = (event) => {
        const data = JSON.parse(event.data)
        ask_data.value = data.a.map(x => ({
            price: x[0],
            volume: x[1]
        }))

        bid_data.value = data.b.map(x => ({
            price: x[0],
            volume: x[1]
        }))
        console.log(ask_data.value)
    }
</script>

<template>
    <div id="order_book">
        <div id="order_book_element">
            <div id="exchange_name">Bybit</div>
            <div id="order_book_labels">
                <div>
                    <span>Цена</span>
                    <div id="separator">
                        <div id="order_book_prices" v-for="(row, i) in ask_data.splice(0, 5)" :key="i">
                            <div id="sell_label">{{ row.price  }}</div>
                        </div>

                        <div id="mid_price"></div>
                    </div>
                    <div id="separator">
                        <div id="order_book_prices" v-for="(row, i) in bid_data.splice(0, 5)" :key="i">
                            <div id="buy_label">{{ row.price }}</div>
                        </div>
                    </div>
                </div>
                
                <div>
                    <div>
                        <span>Обьём$</span>
                        <div id="separator">
                            <div id="order_book_prices" v-for="(row, i) in ask_data.splice(0, 5)" :key="i">
                                <div id="sell_label">{{ row.volume }}</div>
                            </div>
                            <div id="mid_price"></div>
                        </div>

                        <div id="separator">
                            <div id="order_book_prices" v-for="(row, i) in ask_data.splice(0, 5)" :key="i"> 
                                <div id="buy_label">{{ row.volume }}</div> 
                            </div>
                        </div>
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

    #order_book_prices {
        margin-top: 10px;
    }

    #buy_label, #sell_label {
        margin-top: 10px;
    }

    #buy_label {
        color: green;
        /* box-shadow: 1px 1px 80px green; */
    }

    #sell_label {
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