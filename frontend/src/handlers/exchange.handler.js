export function exchange_handler(
    event_data,
    configStore
) {
    const handlers = {
        update_exchange: (data) => configStore.updateExchange(data),
        add_exchange: (data) => configStore.addExchange(data),
        default: (data) => {
            console.log(`Неизвестное событие: ${data}`)
        }
    }

    const handler = handlers[event_data.payload.event] || handlers.default
    handler(event_data)

            // } else if (event_data.payload.event == "add_exchange") {
            //   // Добавляем биржу
            //   userStateStore.exchanges.push({
            //     id: userStateStore.exchanges.length + 1,
            //     name: event_data.payload.exchange_name,
            //     is_available: true
            //   })
            // }
}