<script setup>
    const order_book_data = {
        exchange1: {
            name: "Bybit",
            order_book: {
                prices: {
                    sell: [
                        0.0918, 0.0917, 0.0916, 0.0915, 0.0914, 0.0913,
                    ]
                },
                volumes: {
                    sell: [
                        16681, 12529, 9540, 6506, 3456, 354.84,
                    ]
                }
            }
        },  
        exchange2: {
            name: "Mexc",
            order_book: {
                prices: {
                    sell: [
                        0.1029, 0.1028, 0.1027, 0.1026, 0.1026, 0.1024,
                    ]
                },
                volumes: {
                    sell: [
                        45781, 43059, 35649, 27918, 19544, 10075,
                    ]
                }
            }
        } 
    }
</script>

<template>
    <div id="order_book">
        <div id="order_book_element" v-for="data in order_book_data" :key="data">
            <div id="exchange_name">{{ data.name }}</div>
            <div id="order_book_labels">
                <div>
                    <span>Цена</span>
                    <div id="order_book_prices" v-for="sell in data.order_book.prices.sell" :key="sell">
                        <div id="sell_label">{{ sell }}</div>
                    </div>
                </div>
                <div>
                    <div>
                        <span>Обьём $</span>
                        <div id="order_book_prices" v-for="sell_volume in data.order_book.volumes.sell" :key="sell_volume">
                            <div id="sell_label">{{ sell_volume }}</div>
                        </div>
                    </div>
                </div>
            </div>
        </div>

        <!-- <div id="order_book_element">
            <div id="exchange_name">{{ exchangeName2 }}</div>
            <div id="order_book_labels">
                <div>
                    <span>Цена</span>
                    <div id="order_book_prices">
                        <div id="sell_label">0.1029</div>
                        <div id="sell_label">0.1028</div>
                        <div id="sell_label">0.1027</div>
                        <div id="sell_label">0.1026</div>
                        <div id="sell_label">0.1026</div>
                        <div id="sell_label">0.1024</div>
                    </div>
                </div>
                <div>
                    <div>
                        <span>Обьём $</span>
                        <div id="order_book_prices">
                            <div id="sell_label">45,781</div>
                            <div id="sell_label">43,059</div>
                            <div id="sell_label">35,649</div>
                            <div id="sell_label">27,918</div>
                            <div id="sell_label">19,544</div>
                            <div id="sell_label">10,075</div>
                        </div>
                    </div>
                </div>
            </div>
        </div> -->
    </div>
</template>

<style scoped>
    #order_book {
        margin-top: 20px;
        display: flex;
        gap: 20px;
    }

    #order_book_element {
        flex: 1;
        width: 100%;
        padding: 15px;
        border: 1px solid #31312ddc;
        color: #ffffff;
        font-weight: 600;
        border-radius: 8px;
        font-size: 18px;
        margin-top: 10px;
        background-color: #121212;
        outline: 1px solid #31312ddc;
        box-sizing: border-box;
        cursor: pointer;
        text-transform: uppercase;
        text-align: end;
    }

    #exchange_name {
        text-align: center;
        text-transform: capitalize;
        font-size: 18px;
    }

    #order_book_labels {
        display: flex;
        justify-content: space-evenly;
        gap: 20px;
        text-transform: capitalize;
        font-size: 18px;
    }

    #order_book_prices {
        margin-top: 10px;
    }

    #buy_label, #sell_label {
        margin-top: 10px;
    }

    #buy_label {
        color: green;
    }

    #sell_label {
        color: rgb(151, 15, 15);
    }
</style>