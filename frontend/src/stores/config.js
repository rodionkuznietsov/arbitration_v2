import { defineStore } from "pinia";

export const useConfigStore = defineStore("configStore", {
    state: () => ({
        exchanges: [],
        marketTypes: ["Спот", "Фьючерс"]
    }),

    actions: {
        updateExchange(event_data) {
            if (event_data.payload.is_available == false) {
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
                    this.__insert_or_not__(event_data.payload.exchange_name)
                }
            }
        },

        __insert_or_not__(
            value
        ) {
            if (!this.exchanges.includes(value)) {
                this.exchanges.push(value)
            }
        },
        
        addExchange(exchange_data) {
            this.__insert_or_not__(exchange_data.payload.exchange_name)
        },

        clearExchanges() {
            this.exchanges = []
        }
    }
})