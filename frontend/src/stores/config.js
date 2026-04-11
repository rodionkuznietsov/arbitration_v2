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
                    e => e !== event_data.payload.exchange_name
                )
            } else {
                const index = this.exchanges.findIndex(
                    ex => ex === event_data.payload.exchange_name
                )

                // Добавляем биржу
                if (index === -1) {
                    if (!this.exchanges.includes(event_data.payload.exchange_name)) {
                        this.exchanges.push(event_data.payload.exchange_name)
                    }
                }
            }
        },
        
        addExchange(exchange_data) {
            alert(`AddExchange: ${JSON.stringify(exchange_data)}`)
        }
    }
})