// import { useOrderBookStore } from "@/stores/orderbook"

// const orderBookStore = useOrderBookStore()

export function handle_websocket_data(
    event_data,
    userStateStore,
    orderBookStore
) {
    userStateStore.changeStatus(event_data.payload.status)
    userStateStore.isBotRunning = event_data.payload.isBotRunning

    // Устанавливаем валидные данные для отображения стакана
    if (userStateStore.isBotRunning) {
            orderBookStore.updateHeader(
            event_data.payload.symbol,
            event_data.payload.longExchange,
            event_data.payload.longOrderType,
            event_data.payload.shortExchange,
            event_data.payload.shortOrderType
        )
    }
    
    if (event_data.ws_data.channel == "order_book") {
        orderBookStore.longLastPrice = event_data.ws_data.result.data.order_book.long.last_price
        orderBookStore.shortLastPrice = event_data.ws_data.result.data.order_book.short.last_price
        
        orderBookStore.longAsks = event_data.ws_data.result.data.order_book.long.asks
        orderBookStore.longBids = event_data.ws_data.result.data.order_book.long.bids
        
        orderBookStore.shortAsks = event_data.ws_data.result.data.order_book.short.asks
        orderBookStore.shortBids = event_data.ws_data.result.data.order_book.short.bids
    }
}