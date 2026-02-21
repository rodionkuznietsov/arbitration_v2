import { defineStore } from "pinia";

export const useChartStore = defineStore('chart', {
    state: () => ({
        finished: false,
        lastShortLine: {},
        lastLongLine: {},
        percision: 100000,
        minMove: 0.0000000000001
    }),
    actions: {
        
    }
})