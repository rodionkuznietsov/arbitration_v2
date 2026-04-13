export function user_state_handler(
    event_data,
    userStateStore,
    orderBookStore,
    configStore
) {
    const handlers = {
        init_data: (data) => userStateStore.set_init_data(data.payload, orderBookStore),
        exchange_invalidated: (data) => userStateStore.set_exchange(data.payload.data),
        default: (data) => {
            console.log(`Неизвестное событие: ${data}`)
        }
    }

    configStore
    const handler = handlers[event_data.payload.event] || handlers.default
    handler(event_data)

    // userStateStore.symbol = event_data.payload.symbol
    // userStateStore.longExchange = event_data.payload.longExchange != "unknown" ? event_data.payload.longExchange : configStore.exchanges[0] ? configStore.exchanges[0] : "Нет доступной биржи"
    // userStateStore.longOrderType = event_data.payload.longOrderType 

    // userStateStore.shortExchange = event_data.payload.shortExchange != "unknown" ? event_data.payload.shortExchange : configStore.exchanges[1] ? configStore.exchanges[1] : userStateStore.longExchange
    // userStateStore.shortOrderType = event_data.payload.shortOrderType
}