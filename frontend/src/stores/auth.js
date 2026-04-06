import { defineStore } from "pinia";

export const useAuthStore = defineStore('authStore', {
    state: () => ({
        tg_user_id: 708748005,
        success: true
    })
})