import { defineStore } from "pinia";

export const useAuthStore = defineStore('authStore', {
    state: () => ({
        data: null,
        success: false
    })
})