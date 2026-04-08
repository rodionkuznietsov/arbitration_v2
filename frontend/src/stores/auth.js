import { defineStore } from "pinia";

export const useAuthStore = defineStore('authStore', {
    state: () => ({
        token: null,
        tg_user_id: 0,
        success: true
    })
})