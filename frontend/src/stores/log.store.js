import { defineStore } from "pinia";

export const useLogStore = defineStore('logStore', {
    state: () => ({
        logs: []
    }),

    actions: {
        addLog(log) {
            alert(log)
            alert(this.logs)

            this.logs.push(log)
        },

        sortLogs() {
            this.logs.sort((a, b) => b.timestamp - a.timestamp)
        }
    }   
})