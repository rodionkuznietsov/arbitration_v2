export function exchange_handler(
    event_data,
    configStore
) {
    const handlers = {
        update_exchange: (data) => configStore.updateExchange(data),
        add_exchange: (data) => configStore.addExchange(data),
        clear_exchanges: () => configStore.clearExchanges(),
        default: (data) => {
            console.log(`Неизвестное событие: ${data}`)
        }
    }

    const handler = handlers[event_data.payload.event] || handlers.default
    handler(event_data)
}