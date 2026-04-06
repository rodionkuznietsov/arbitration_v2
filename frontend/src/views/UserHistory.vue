<script setup>
    import { useAuthStore } from '@/stores/auth';
    import { API_URL, getBotLogs } from '@/utils/logFetch';
    import { onMounted, ref } from 'vue';

    const authStore = useAuthStore()

    let logs = ref([])
        
    onMounted(async () => {
        const logsHistory = await getBotLogs(authStore.tg_user_id)
        logs.value = logsHistory
        
        // Подписываемся на получение новых логов
        const es = new EventSource(`${API_URL}/subscribe/logs/${authStore.tg_user_id}`)

        es.onmessage = (event) => {
            const newLog = JSON.parse(event.data)
            logs.value.push(newLog)
            logs.value.sort((a, b) => b.timestamp - a.timestamp) // DESC
        }

        es.onerror = (err) => {
            console.error("SSE ошибка", err);
            es.close(); // или повторное соединение через setTimeout
        };
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
            <div class="logs">
                <div class="log-table" v-for="(v, index) in logs" :key="index">
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
                    <div class="log-table-element">Событие: {{ v.event.toUpperCase() }}</div>
                    <div class="log-table-element">Тикер: {{ v.symbol.toUpperCase() }}USDT</div>
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

</style>