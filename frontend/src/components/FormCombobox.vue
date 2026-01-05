<script setup>
    import { ref, defineProps } from "vue"

    const props = defineProps({
        options: Array
    })

    const show = ref(false)
    const exchange = ref("Bybit")

    const select = (value) => {
        exchange.value = value
        show.value = false
    }
</script>

<template>
<div>
    <div class="combo">
        <input id="combobox" :value="exchange" readonly="true" @click="show = true">
    </div>

    <ul class="combobox-list" id="optionsList" v-show="show">
        <li id="combobox_element" v-for="opt in props.options" :key="opt" @mousedown="select(opt)">{{ opt }}</li>
    </ul>
</div>
</template>

<style scoped>
#combobox { 
  width: 100%;
  padding: 15px;
  border: 1px solid #54555a;
  color: #ffffff;
  font-weight: 600;
  border-radius: 8px;
  font-size: 20px;
  margin-top: 10px;
  background-color: #23262b;
  outline: 1px solid #65666d;
  box-sizing: border-box;
  cursor: pointer;
  text-transform: uppercase;
}

.combo {
 position: relative;
 display: inline-block; 
}

.combo::after {
  content: "➤";
  position: absolute;
  transform: translateY(100%) rotate(90deg);
  right: 20px;
}

.combobox-list {
  background-color: #23262b;
  border-radius: 8px;
  padding: 15px;
}

#combobox_element {
  cursor: pointer;
  padding: 5px;
  border-radius: 8px;
  list-style: none;
  margin: 0;
  padding-left: 0;
  text-align: left;
}

#combobox_element:hover {
  filter: opacity(75%);
  background-color: #54555a;
}
</style>