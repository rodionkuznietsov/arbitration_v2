<script setup>
import { ref } from 'vue';


    const key = ref('')

    const submitForm = async () => {
        try {
            const formData = new FormData()
            formData.append('key', key.value)
            console.log(key.value)

            const response = await fetch('https://unfarming-untethered-flynn.ngrok-free.dev/api/user/update', {
                method: 'POST',
                body: formData
            })

            const result = await response.json()
            console.log(result)
        } catch(err) {
            console.log("Ошибка:", err)
        }
    }
</script>

<template>
    <div class="header">
        <div class="title">Насстройка бирж</div>
        <div class="warning">!!! Рекомендуем эту страницу никому не показывать. !!!</div>
    </div>
    <form @submit.prevent="submitForm" id="form_column">
        <div>
            <label for="bybit">Bybit</label>
            <input v-model="key" class="form_input" type="text" name="key" id="bybit" placeholder="Api Key">
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

        <input class="default-btn default-button-margin" type="submit" value="Сохранить">
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