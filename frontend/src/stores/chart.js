import { defineStore } from "pinia";

export const useChartStore = defineStore('chart', {
    state: () => ({
        finished: false,
        lastShortLine: {},
        lastLongLine: {},
    }),
    actions: {
        
    }
})