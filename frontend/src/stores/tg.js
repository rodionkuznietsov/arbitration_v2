import { defineStore } from "pinia";

export const useTgStore = defineStore("tg", {
    state: () => ({
        tgObject: null
    })
})