export function user_state_handler(
    event_data,
    userStateStore,
    orderBookStore,
    configStore
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
    userStateStore.longExchange = event_data.payload.longExchange != "unknown" ? event_data.payload.longExchange : configStore.exchanges[0] ? configStore.exchanges[0].name : "Нет доступной биржи"
    userStateStore.longOrderType = event_data.payload.longOrderType 

    userStateStore.shortExchange = event_data.payload.shortExchange != "unknown" ? event_data.payload.shortExchange : configStore.exchanges[1] ? configStore.exchanges[1].name : userStateStore.longExchange
    userStateStore.shortOrderType = event_data.payload.shortOrderType
}