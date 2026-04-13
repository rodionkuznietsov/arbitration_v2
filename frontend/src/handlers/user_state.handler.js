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
}