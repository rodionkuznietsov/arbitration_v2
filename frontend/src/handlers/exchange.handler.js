export function exchange_handler(
    event_data,
    // configStore
) {
    // const handlers = {
    //     update_exchange: (data) => configStore.updateExchange(data),
    //     add_exchange: (data) => configStore.addExchange(data),
    //     default: (data) => {
    //         console.log(`Неизвестное событие: ${data}`)
    //     }
    // }

    // const handler = handlers[event_data.payload.event] || handlers.default
    // handler(event_data)

    alert(JSON.stringify(event_data))
    // if (event_data.payload.event == "update_exchange") {
    //     if (!event_data.payload.is_available) {
    //         // Удаляем биржу
    //         userStateStore.exchanges = userStateStore.exchanges.filter(
    //             ex => ex.name !== event_data.payload.exchange_name
    //         )

    //         if (userStateStore.exchanges.length > 0) {
    //             homeStore.longExchange = userStateStore.exchanges[0].name
    //             homeStore.shortExchange = userStateStore.exchanges[0].name
    //         } else {
    //             homeStore.longExchange = "Нет доступных бирж"
    //             homeStore.shortExchange = "Нет доступных бирж"
    //         }
    //         } else {
    //         const index = userStateStore.exchanges.findIndex(
    //             ex => ex.name === event_data.payload.exchange_name
    //         )

    //         // Добавляем биржу
    //         if (index === -1) {
    //             userStateStore.exchanges.push({
    //             id: userStateStore.exchanges.length + 1,
    //             name: event_data.payload.exchange_name,
    //             is_available: true
    //             })
    //         }

    //         if (userStateStore.exchanges.length > 1) {
    //             homeStore.longExchange = userStateStore.exchanges[1].name
    //             homeStore.shortExchange = userStateStore.exchanges[0].name
    //         } else {
    //             homeStore.longExchange = userStateStore.exchanges[0].name
    //             homeStore.shortExchange = userStateStore.exchanges[0].name
    //         }
    //     }
    // }
            // } else if (event_data.payload.event == "add_exchange") {
            //   // Добавляем биржу
            //   userStateStore.exchanges.push({
            //     id: userStateStore.exchanges.length + 1,
            //     name: event_data.payload.exchange_name,
            //     is_available: true
            //   })
            // }
}