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
    userStateStore.longExchange = event_data.payload.longExchange != "unknown" ? event_data.payload.longExchange : "Нет доступной биржи"
    userStateStore.longOrderType = event_data.payload.longOrderType 

    userStateStore.shortExchange = event_data.payload.shortExchange != "unknown" ? event_data.payload.shortExchange : "Нет доступной биржи"
    userStateStore.shortOrderType = event_data.payload.shortOrderType

}