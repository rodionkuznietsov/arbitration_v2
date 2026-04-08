import { defineStore } from "pinia";

export const useHomeStore = defineStore('homeStore', {
    state: () => ({
        longExchange: "Нет доступных бирж",
        shortExchange: "Нет доступных бирж"
    })
})