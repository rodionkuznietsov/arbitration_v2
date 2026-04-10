export function log_handler(
    event_data,
    tgStore,
    logStateStore
) {
    try {
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