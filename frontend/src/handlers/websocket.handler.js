export function ws_handler(
    event_data,
    orderBookStore
) { 
    if (event_data.payload.channel == "order_book") {
        orderBookStore.longLastPrice = event_data.payload.result.data.order_book.long.last_price
        orderBookStore.shortLastPrice = event_data.payload.result.data.order_book.short.last_price
        
        orderBookStore.longAsks = event_data.payload.result.data.order_book.long.asks
        orderBookStore.longBids = event_data.payload.result.data.order_book.long.bids
        
        orderBookStore.shortAsks = event_data.payload.result.data.order_book.short.asks
        orderBookStore.shortBids = event_data.payload.result.data.order_book.short.bids
    }
}