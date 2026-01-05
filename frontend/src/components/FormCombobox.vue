<script setup>
    import { ref, defineProps, defineEmits, onMounted, onUnmounted } from "vue"

    const props = defineProps({
        placeholder: String,
        options: Array
    })

    const show = ref(false)
    const arrow_class = ref("arrow")
    const emit = defineEmits(['update:placeholder'])
    const localPlaceholder = ref(props.placeholder)

    console.log('placeholder:', props.placeholder)

    const select = (value) => {
        emit('update:placeholder', value)
        show.value = false
        arrow_class.value = "arrow"
        localPlaceholder.value = value
    }

    const inside_div = ref({x: 0.0, y: 0.0})

    function getPosInsideDiv(e) {
      inside_div.value = {x: e.clientX, y: e.clientY}
    }

    function getPosInsideMouseClick(e) {
      const x = e.clientX
      const y = e.clientY

      if (inside_div.value.x != x && inside_div.value.y != y) {
        show.value = false
        arrow_class.value = "arrow"
      } 
    }

    onMounted(() => {
      document.addEventListener('click', getPosInsideMouseClick)
    })

    onUnmounted(() => {
      document.removeEventListener('click', getPosInsideMouseClick)
    })

</script>

<template>
  <div @click="getPosInsideDiv">
      <div :class="arrow_class">
          <input id="combobox" :value="localPlaceholder" readonly="true" @click="show = true; arrow_class = 'arrow_rotate'">
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
  border: 1px solid #31312ddc;
  color: #ffffff;
  font-weight: 600;
  border-radius: 8px;
  font-size: 18px;
  margin-top: 10px;
  background-color: #121212;
  outline: 1px solid #31312ddc;
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
  transition: all 1s;
}

.arrow_rotate::after {
  content: "➤";
  position: absolute;
  transform: translateY(100%) rotate(-90deg);
  right: 20px;
  transition: all 1s;
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
  transition: background 0.6s;
} 
</style>