export function log_handler(
    event_data,
    tgStore,
    logStateStore
) {
    try {
        // userStateStore.symbol = event_data.payload.symbol.replace("USDT", ""),
        // homeStore.longExchange = event_data.payload.longExchange,
        // userStateStore.longOrderType = event_data.payload.longOrderType,
        // homeStore.shortExchange = event_data.payload.shortExchange,
        // userStateStore.shortOrderType = event_data.payload.shortOrderType

        const log = {
            event: event_data.payload.event,
            symbol: event_data.payload.symbol,
            long_exchange: event_data.payload.long_exchange,
            short_exchange: event_data.payload.short_exchange,
            timestamp: event_data.timestamp
        }

        logStateStore.addLog(log)
        logStateStore.sortLogs()
    } catch(err) {
        tgStore.tgObject.showAlert("Ошибка загрузки страницы", () => {
        console.log("Пользователь закрыл alert")
        });
    }
}