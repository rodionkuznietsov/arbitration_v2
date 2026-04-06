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
            <div class="view_header">
                <div class="title">Моя история</div>
            </div>
            <div class="logs">
                <div class="log" v-for="(v, index) in logs" :key="index">
                    <div>{{ index }}</div>
                    <div class="log-event">{{ v.event }}</div>
                    <div class="log-timestamp">{{ new Date(v.timestamp * 1000).toLocaleDateString() }} {{ new Date(v.timestamp * 1000).toLocaleTimeString() }}</div>
                </div>
            </div>
        </div>
    </div>
</template>

<style scoped>
    .logs {
        margin-bottom: var(--default-margin-bottom);
    }

    .log-description {
        display: flex;
        flex-direction: row;
        justify-content: space-between;
        margin-top: var(--default-margin-top);
    }

    .log {
        display: flex;
        flex-direction: column;
        background-color: var(--default-logs-bg);
        border-radius: var(--default-border-radius);
        padding: var(--default-padding);
        width: 100%;
        box-sizing: border-box;
        margin: 0 0 10px;
    }

</style>