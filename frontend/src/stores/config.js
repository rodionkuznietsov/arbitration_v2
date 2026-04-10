import { defineStore } from "pinia";

export const useConfigStore = defineStore("configStore", {
    state: () => ({
        exchanges: [],
        marketTypes: ["Спот", "Фьючерс"]
    }),

    actions: {
        updateExchange(exchange_data) {
            alert(`UpdateExchange: ${JSON.stringify(exchange_data)}`)
        },
        
        addExchange(exchange_data) {
            alert(`AddExchange: ${JSON.stringify(exchange_data)}`)
        }
    }
})