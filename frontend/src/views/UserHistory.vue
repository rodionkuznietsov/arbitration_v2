<template>
    <div class="scroll">
        <div class="view_header">
            <div class="title">Моя история</div>
        </div>
        <div class="logs">
            <div class="log-year">Год 2026</div>
            <div class="log">
                <div class="log-content">
                    <div class="log-title">
                        <i>4 Апреля в 23:15:23 -> Запуск бота с тикером: BTCUSDT</i>
                    </div>
                    <div class="log-description">
                        <div>
                            <i>Gate - Спот (Лонг)</i>
                        </div>
                        <div>
                            <i>Bybit - Спот (Шорт)</i>
                        </div>
                    </div>
                    <div class="log-status">Успешно</div>
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
    }

    .log-title {
        display: flex;
        flex-direction: row;
        justify-content: space-between;
    }
</style>