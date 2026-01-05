<script setup>
    import { ref, defineProps } from "vue"

    const props = defineProps({
        options: Array
    })

    const show = ref(false)
    const exchange = ref("Bybit")
    const arrow_rotate = ref("arrow")

    const select = (value) => {
        exchange.value = value
        show.value = false
        arrow_rotate.value = "arrow"
    }
</script>

<template>
<div>
    <div :class="arrow_rotate">
        <input id="combobox" :value="exchange" readonly="true" @click="show = true; arrow_rotate = 'arrow_rotate'">
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

.arrow, .arrow_rotate {
    position: relative;
    display: inline-block; 
    width: 100%;
}

.arrow::after {
  content: "➤";
  position: absolute;
  transform: translateY(100%) rotate(90deg);
  right: 20px;
}

.arrow_rotate::after {
  content: "➤";
  position: absolute;
  transform: translateY(100%) rotate(-90deg);
  right: 20px;
  transition: all 0.5ms;
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