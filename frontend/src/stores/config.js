import { defineStore } from "pinia";

export const useConfigStore = defineStore("configStore", {
    state: () => ({
        exchanges: [],
        marketTypes: ["Спот", "Фьючерс"]
    }),

    actions: {
        updateExchange(event_data) {
            if (!event_data.payload.is_available) {
                // Удаляем биржу
                this.exchanges = this.exchanges.filter(
                    ex => ex.name !== event_data.payload.exchange_name
                )
            } else {
                const index = this.exchanges.findIndex(
                    ex => ex.name === event_data.payload.exchange_name
                )

                // Добавляем биржу
                if (index === -1) {
                    this.exchanges.push({
                        id: this.exchanges.length + 1,
                        name: event_data.payload.exchange_name,
                        is_available: true
                    })
                }
            }
        },
        
        addExchange(exchange_data) {
            alert(`AddExchange: ${JSON.stringify(exchange_data)}`)
        }
    }
})