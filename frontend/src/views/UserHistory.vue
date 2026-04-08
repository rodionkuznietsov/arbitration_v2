<script setup>
    import { useAuthStore } from '@/stores/auth';
    import { useUserState } from '@/stores/user_state';
    import { getBotLogs } from '@/utils/logFetch';
    import { onMounted } from 'vue';
    import FixedTransactions from '@/components/history/FixedTransactions.vue'

    const authStore = useAuthStore()
    const userStateStore = useUserState()

    onMounted(async () => {
        const logsHistory = await getBotLogs(authStore.token)
        userStateStore.logs = logsHistory
    })
</script>

<template>
    <div class="page">
        <div class="scroll">
            <div class="view_header" style="
                display: flex;
                flex-direction: row;
                justify-content: space-between;
                align-items: center;
            ">
                <div class="title">Моя история</div>
                <div class="default-btn filter_btn">Фильтры</div>
            </div>
            <FixedTransactions />
            <div class="logs">
                <div class="log-table" v-for="(v, index) in userStateStore.logs" :key="index">
                    <div style="padding: 8px;">
                    {{ 
                        new Date(v.timestamp * 1000)
                            .toLocaleDateString(
                                'ru-Ru', {
                                    day: 'numeric',
                                    month: 'long',
                                    year: 'numeric'
                                }
                            ) 
                    }} </div>
                    <div class="log-table-element">ID: {{ index + 1 }}</div>
                    <div class="log-table-element">Событие: <span class="bold-text">{{ v.event.toUpperCase() }}</span></div>
                    <div class="log-table-element">Тикер: {{ v.symbol.toUpperCase() }}</div>
                    <div class="log-table-element">Лонг биржа: {{ v.long_exchange }}</div>
                    <div class="log-table-element">Шорт биржа: {{ v.short_exchange }}</div>
                    <div class="log-table-element">Время: {{ new Date(v.timestamp * 1000).toLocaleTimeString() }}</div>
                </div>
            </div>
        </div>
    </div>
</template>

<style scoped>
    .filter_btn {
        flex: 0;
    }

    .logs {
        margin-bottom: var(--default-margin-bottom);
    }

    .log-table {
        border-radius: var(--default-border-radius);
        width: 100%;
        box-sizing: border-box;
        text-transform: capitalize;
        table-layout: fixed;
        border-collapse: collapse;
        text-align: left;
        background-color: var(--log-table-bg-color);
    }

    .log-table-element {
        color: var(--default-font-color);
        white-space: nowrap;
        word-break: break-word;
        overflow: auto;
        text-overflow: clip;
        padding: 8px;
        background-color: #fff;
        margin: 0 0 5px;
    }

    .bold-text {
        font-weight: 700;
    }

</style>