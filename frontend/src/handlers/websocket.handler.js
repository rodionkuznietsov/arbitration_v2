export function ws_handler(
    event_data,
    orderBookStore
) { 
    if (event_data.ws_data.channel == "order_book") {
        orderBookStore.longLastPrice = event_data.ws_data.result.data.order_book.long.last_price
        orderBookStore.shortLastPrice = event_data.ws_data.result.data.order_book.short.last_price
        
        orderBookStore.longAsks = event_data.ws_data.result.data.order_book.long.asks
        orderBookStore.longBids = event_data.ws_data.result.data.order_book.long.bids
        
        orderBookStore.shortAsks = event_data.ws_data.result.data.order_book.short.asks
        orderBookStore.shortBids = event_data.ws_data.result.data.order_book.short.bids
    }
}