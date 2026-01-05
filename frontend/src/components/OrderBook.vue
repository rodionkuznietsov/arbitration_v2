<script setup>
    const order_book_data = {
        exchange1: {
            name: "Bybit",
            order_book: {
                prices: {
                    sell: [
                        0.0918, 0.0917, 0.0916, 0.0915, 0.0914, 0.0913,
                    ],
                    buy: [
                        0.0912, 0.0911, 0.0910, 0.0909, 0.0908, 0.0907, 
                    ]
                },
                volumes: {
                    sell: [
                        16681, 12529, 9540, 6506, 3456, 354.84,
                    ],
                    buy: [
                        67919, 4653, 6788, 8651, 11826, 14611
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
                    ],
                    buy: [
                        0.1023, 0.1022, 0.1021, 0.1020, 0.1019, 0.1018,
                    ]
                },
                volumes: {
                    sell: [
                        45781, 43059, 35649, 27918, 19544, 10075,
                    ],
                    buy: [
                        1371, 8586, 16414, 26018, 36150, 42231
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
                    <div id="separator">
                        <div id="order_book_prices" v-for="sell in data.order_book.prices.sell" :key="sell">
                            <div id="sell_label">{{ sell }}</div>
                        </div>

                        <div id="mid_price">0.0913</div>
                    </div>
                    
                    <div id="separator">
                        <div id="order_book_prices" v-for="buy in data.order_book.prices.buy" :key="buy">
                            <div id="buy_label">{{ buy }}</div>
                        </div>
                    </div>
                </div>
                
                <div>
                    <div>
                        <span>Обьём$</span>
                        <div id="separator">
                            <div id="order_book_prices" v-for="sell_volume in data.order_book.volumes.sell" :key="sell_volume">
                                <div id="sell_label">{{ sell_volume }}</div>
                            </div>
                            <div id="mid_price">(0.10%)</div>
                        </div>

                        <div id="separator">
                            <div id="order_book_prices" v-for="buy_volume in data.order_book.volumes.buy" :key="buy_volume">
                                <div id="buy_label">{{ buy_volume }}</div>
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
    }

    #order_book_element {
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
        overflow-y: auto;
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
        box-shadow: 1px 1px 80px green;
    }

    #sell_label {
        color: rgb(151, 15, 15);
        box-shadow: 1px 1px 80px rgb(151, 15, 15);
    }

    #mid_price {
        margin-top: 20px;
        margin-bottom: 20px;
    }

    #separator {
        margin-bottom: 30px;
    }
</style>