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

    const arrowInput = ref(null)
    const comboboxList = ref(null)

    function popup() {
      if (arrowInput.value) {
        const inputWidth = arrowInput.value.offsetWidth
        if (comboboxList.value) {
          comboboxList.value.style.width = inputWidth + 'px'
        }
      }

      show.value = true; 
      arrow_class.value = 'arrow_rotate'
    }

</script>

<template>
  <div @click="getPosInsideDiv">
      <div :class="arrow_class">
          <input id="combobox" :value="localPlaceholder" readonly="true" @click="popup" ref="arrowInput">
      </div>

      <ul class="combobox-list" id="optionsList" v-show="show" ref="comboboxList">
          <li id="combobox_element" v-for="opt in props.options" :key="opt" @mousedown="select(opt)">{{ opt }}</li>
      </ul>
  </div>
</template>

<style scoped>
#combobox { 
  width: 100%;
  padding: 8px;
  border: 1px solid #303c51;
  color: #ffffff;
  border-radius: 8px;
  font-size: 16px;
  margin-top: 10px;
  background-color: #303c51;
  outline: 1px solid #303c51;
  box-sizing: border-box;
  cursor: pointer;
  text-transform: capitalize;
}

.arrow, .arrow_rotate {
    position: relative;
    display: inline-block; 
    width: 100%;
}

.arrow::after {
  content: "▾";
  position: absolute;
  transform: translateY(75%);
  right: 20px;
  transition: all 1s;
}

.arrow_rotate::after {
  content: "▾";
  position: absolute;
  transform: translateY(100%) rotate(180deg);
  right: 20px;
  transition: all 0.9s;
}

.combobox-list {
  position: absolute;
  background-color: rgba(18, 18, 18, 0.97);
  border-radius: 8px;
  padding: 8px;
  z-index: 9999;
  box-sizing: border-box;
  border: 1px solid #31312ddc;
}

#combobox_element {
  cursor: pointer;
  padding: 8px;
  list-style: none;
  margin: 0;
  text-align: left;
  border-radius: 8px;
}

#combobox_element:hover {
  filter: opacity(75%);
  background-color: #54555a;
  transition: background 0.6s;
} 
</style>