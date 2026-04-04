<template>
    <div class="scroll">
        <div class="view_header">
            <div class="title">Моя история</div>
        </div>
        <div class="logs">
            <div class="log">
                <div class="log-content">
                    <div class="log-title">Запуск бота</div>
                    <div class="log-description">BTC -> Bybit-SPOT & Gate-FUTURE</div>
                </div>
                <div class="log-timestamp">04.04.2026 23:15:23</div>
            </div>
        </div>
    </div>
</template>

<style scoped>
    .logs {
        margin-bottom: var(--default-margin-bottom);
    }

    .log {
        display: flex;
        flex-direction: column;
        background-color: var(--default-logs-bg);
        border-radius: var(--default-border-radius);
        padding: var(--default-padding);
        width: 100%;
        box-sizing: border-box;
    }

    .log-timestamp {
        display: flex;
        flex-direction: row-reverse;
    }
</style>