export function user_state_handler(
    event_data,
    userStateStore,
    orderBookStore
) {
    if (userStateStore.isBotRunning) {
        orderBookStore.updateHeader(
            event_data.payload.symbol,
            event_data.payload.longExchange,
            event_data.payload.longOrderType,
            event_data.payload.shortExchange,
            event_data.payload.shortOrderType
        )
    }

    userStateStore.symbol = event_data.payload.symbol
    userStateStore.longExchange = event_data.payload.longExchange
    userStateStore.longOrderType = event_data.payload.longOrderType 

    userStateStore.shortExchange = event_data.payload.shortExchange
    userStateStore.shortOrderType = event_data.payload.shortOrderType

}