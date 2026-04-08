<template>
    <div style="margin-bottom: 10px;">
        <div class="title">Активные сделки</div>
        <table class="active-order">
            <thead>
                <td>#</td>
                <td>Статус</td>
                <td>Биржи</td>
                <td>Тикер</td>
                <td>Время работы</td>
            </thead>
            <tbody>
                <td>1</td>
                <td id="status"><div class="online" id="status_circle"></div>Активный</td>
                <td>Bybit/Short</td>
                <td>BTCUSDT</td>
                <td>19 Минут</td>
            </tbody>
        </table>
    </div>
</template>

<style scoped>
    .active-order {
        background-color: var(--log-table-bg-color);
        padding: var(--default-padding);
        border-radius: var(--default-border-radius);
        width: 100%;
        box-sizing: border-box;
        text-align: center;
        word-wrap: break-word;
        white-space: normal;
        text-overflow: ellipsis;
    }
</style>