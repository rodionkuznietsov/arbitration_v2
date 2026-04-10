export function log_handler(
    event_data,
    tgStore,
    userStateStore,
    homeStore
) {
    try {
        userStateStore.symbol = event_data.payload.symbol.replace("USDT", ""),
        homeStore.longExchange = event_data.payload.longExchange,
        userStateStore.longOrderType = event_data.payload.longOrderType,
        homeStore.shortExchange = event_data.payload.shortExchange,
        userStateStore.shortOrderType = event_data.payload.shortOrderType

        userStateStore.logs.push({
            event: event_data.payload.event,
            symbol: event_data.payload.symbol,
            long_exchange: event_data.payload.long_exchange,
            short_exchange: event_data.payload.short_exchange,
            timestamp: event_data.timestamp
        })
        userStateStore.logs.sort((a, b) => b.timestamp - a.timestamp) // DESC
    } catch(err) {
        tgStore.tgObject.showAlert("Ошибка загрузки страницы", () => {
        console.log("Пользователь закрыл alert")
        });
    }
}