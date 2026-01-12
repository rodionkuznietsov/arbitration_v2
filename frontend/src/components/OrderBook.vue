<script setup>
    import { ref, defineExpose, defineProps, defineEmits, computed } from 'vue';

    const props = defineProps({
        modelValue: Boolean
    })

    const isVisible = ref("display: none;")
    const emit = defineEmits(["update:modelValue"])
    const isWarningStatus = ref(props.modelValue)

    let websoket = null
    const formData = ref({})

    const longAsks = ref([])
    const longFirstAskPrice = ref(0.0)

    const longBids = ref([])
    const longFirstBidPrice = ref(0.0)

    const longLastPrice = ref(0.0)
    const longExchange = ref('')
    const longExchangeLogo = ref('')
    const longOrderType = ref('')

    const longArrow = computed(() => {
        if (longLastPrice.value == longFirstAskPrice.value) {
            return '⬆'
        } else if (longLastPrice.value == longFirstBidPrice.value) {
            return '⬇'
        } 
        return '⬆'
    })

    const shortAsks = ref([])
    const shortFirstAskPrice = ref(0.0)

    const shortBids = ref([])
    const shortFirstBidPrice = ref(0.0)

    const shortLastPrice = ref(0.0)
    const shortExchange = ref('')
    const shortExchangeLogo = ref('')
    const shortOrderType = ref('')

    const shortArrow = computed(() => {
        if (shortLastPrice.value == shortFirstAskPrice.value) {
            return '⬆'
        } else if (shortLastPrice.value == shortFirstBidPrice.value) {
            return '⬇'
        } 
        return '⬆'
    })

    const currentTicker = ref('')

    function exchanges(
        data
    ) {
        formData.value = data
        longExchange.value = formData.value.exchanges.longExchange
        longExchangeLogo.value = `../assets/icons/${longExchange.value.toLowerCase()}_logo.svg`
        longOrderType.value = formData.value.types.longType

        shortExchange.value = formData.value.exchanges.shortExchange
        shortExchangeLogo.value = `../assets/icons/${shortExchange.value.toLowerCase()}_logo.svg`
        currentTicker.value = formData.value.ticker
        shortOrderType.value = formData.value.types.shortType
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

            // Для стрелочки
            longFirstAskPrice.value = data.book1?.snapshot?.a?.[0]?.[0];
            longFirstBidPrice.value = data.book1?.snapshot?.b?.[0]?.[0];

            shortFirstAskPrice.value = data.book2?.snapshot?.a[0]?.[0];
            shortFirstBidPrice.value = data.book2?.snapshot?.b?.[0]?.[0];
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
        longAsks.value = []
        longBids.value = []
        
        shortAsks.value = []
        shortBids.value = []
        
        isVisible.value = "display: none;"
    }

    function formatCurrency(value) {
        if (typeof value != 'number') {
            return value;
        }

        return new Intl.NumberFormat(
            "en-US", {
                minimumFractionDigits: 2,
                maximumFractionDigits: 3,
            }
        ).format(value)
    }

    defineExpose({ show, start, stop, exchanges, isWarningStatus })
</script>

<template>
    <div class="order_book_title">Книги ордеров</div>
    <div id="stakan">
        <div id="order_book" :style="isVisible">
            <div id="exchange_name">
                <div class="with_img">
                    <img :src="longExchangeLogo" alt="">
                    <span>{{ longExchange }}</span>
                    <span class="ticker_name">{{ currentTicker }} - {{ longOrderType }}</span>
                </div>
            </div>
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
                        <td colspan="2" class="mid_price" tabindex="3">
                            <div class="with_arrow" :class="longArrow == '⬇' ? 'down' : 'up'">
                                <div>{{ longArrow }} </div>
                                <div>{{ formatCurrency(longLastPrice) }}</div>
                            </div>
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
            <div id="exchange_name">
                <div class="with_img">
                    <img :src="shortExchangeLogo" alt="">
                    <span>{{ shortExchange }}</span>
                    <span class="ticker_name">{{ currentTicker }} - {{ shortOrderType }}</span>
                </div>
            </div>
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
                        <td colspan="2" class="mid_price" tabindex="3">
                            <div class="with_arrow" :class="shortArrow == '⬇' ? 'down' : 'up'">
                                <div>{{ shortArrow }} </div>
                                <div>{{ formatCurrency(shortLastPrice) }}</div>
                            </div>
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
    .order_book_title {
        text-align: center;
        margin-top: 10px;
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