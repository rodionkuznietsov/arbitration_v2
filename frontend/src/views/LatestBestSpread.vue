<script setup>
</script>

<template>
    <div class="page">
        <div class="scroll">
            <div class="title">Топ спредов за 24 часа 🔥</div>
            <div class="spreads">
                <table class="spread-table" style="text-align: center;">
                    <tbody>
                        <tr class="spread-table-header">
                            <td>ID</td>
                            <td>Иконка</td>
                            <td>Биржа</td>
                            <td>Символ</td>
                            <td>Цена</td>
                            <td>Спред</td>
                        </tr>
                        <tr class="spread-table-header">
                            <td>1</td>
                            <td>
                                <img class="icon_svg svg_margin" src="../assets/icons/bybit_logo.svg" alt="">
                                <img class="icon_svg svg_margin" src="../assets/icons/gate_io_logo.svg" alt="">
                            </td>
                            <td>Bybit/Gate</td>
                            <td>AVAXUSDT</td>
                            <td>0.372 / 0.25</td>
                            <td>0.5% / -0.5%</td>
                        </tr>
                    </tbody>
                </table>

                <table class="spread-table" style="text-align: center;">
                    <tbody>
                        <tr class="spread-table-header">
                            <td>ID</td>
                            <td>Иконка</td>
                            <td>Биржа</td>
                            <td>Символ</td>
                            <td>Цена</td>
                            <td>Спред</td>
                        </tr>
                        <tr class="spread-table-header">
                            <td>2</td>
                            <td>
                                <img class="icon_svg svg_margin" src="../assets/icons/bybit_logo.svg" alt="">
                                <img class="icon_svg svg_margin" src="../assets/icons/gate_io_logo.svg" alt="">
                            </td>
                            <td>Bybit/Gate</td>
                            <td>AVAXUSDT</td>
                            <td>0.372 / 0.25</td>
                            <td>0.5% / -0.5%</td>
                        </tr>
                    </tbody>
                </table>
            </div>
        </div>
    </div>
</template>

<style scoped>
    .spreads {
        display: flex;
        flex-direction: column;
        gap: 10px;
    }

    .spread-table {
        background-color: var(--latest-best-spread-table-bg-color);
    }

    .spread-table-header {
        background-color: var(--latest-best-spread-table-header-bg-color);
    }
</style>