<script setup>

</script>

<template>
    <div class="header">
        <div class="title">Насстройка бирж</div>
        <div class="warning">!!! Рекомендуем эту страницу никому не показывать. !!!</div>
    </div>
    <form action="" id="form_column">
        <div>
            <label for="bybit">Bybit</label>
            <input class="form_input" type="text" name="" id="bybit" placeholder="Api Key">
            <input class="form_input" type="text" name="" id="bybit" placeholder="Api Secret">
        </div>

        <div>
            <label for="binance">Binance</label>
            <input class="form_input" type="text" name="" id="binance" placeholder="Api Key">
            <input class="form_input" type="text" name="" id="bybit" placeholder="Api Secret">
        </div>

        <div>
            <label for="kucoin">KuCoin</label>
            <input class="form_input" type="text" name="" id="kucoin" placeholder="Api Key">
            <input class="form_input" type="text" name="" id="bybit" placeholder="Api Secret">
        </div>

        <div>
            <label for="binx">BinX</label>
            <input class="form_input" type="text" name="" id="binx" placeholder="Api Key">
            <input class="form_input" type="text" name="" id="bybit" placeholder="Api Secret">
        </div>

        <div>
            <label for="mexc">Mexc</label>
            <input class="form_input" type="text" name="" id="mexc" placeholder="Api Key">
            <input class="form_input" type="text" name="" id="bybit" placeholder="Api Secret">
        </div>

        <div>
            <label for="gate">Gate</label>
            <input class="form_input" type="text" name="" id="gate" placeholder="Api Key">
            <input class="form_input" type="text" name="" id="bybit" placeholder="Api Secret">
        </div>

        <div>
            <label for="lbank">LBank</label>
            <input class="form_input" type="text" name="" id="lbank" placeholder="Api Key">
            <input class="form_input" type="text" name="" id="bybit" placeholder="Api Secret">
        </div>

        <input class="default-btn default-button-margin" type="button" value="Сохранить">
    </form>
</template>

<style scoped>
    .header {
        padding: var(--default-padding);
        font-size: var(--default-font-size);
    }

    .title {
        color: var(--default-font-color);
    }

    .warning {
        color: #ff7300;
        font-size: 12px;
    }

    #form_column {
        display: flex;
        flex-direction: column;
        gap: var(--default-gap);
        padding: var(--default-padding);
    }
</style>