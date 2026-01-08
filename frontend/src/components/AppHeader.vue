<script setup>
  import { defineExpose, ref } from 'vue';
  import soundOn from '../assets/icons/sound_on.svg'
  import soundOff from '../assets/icons/sound_off.svg'
  import prompt from '../assets/icons/prompt.svg'

  const workStatus = ref(false)
  const img_sound = ref(null)
  const isOn = ref(false)

  function change_work_status(status) {
    workStatus.value = status
  }

  function is_sound_on() {
    if (img_sound.value) {
      isOn.value = !isOn.value
      img_sound.value.src = isOn.value ? soundOn : soundOff
    }
  }

  defineExpose({ change_work_status })
</script>

<template>
    <div id="header">
      <div id="status">
        <div id="status_circle" :class="workStatus == 'online' ? 'online' : workStatus == 'warning' ? 'warning' : 'offline'"></div>
        <span>{{ workStatus == 'online' ? 'Онлайн' : workStatus == 'warning' ? 'Неполадки' : 'Офлайн' }}</span>
        <img class="status_img" :src="workStatus == 'online' ? 'Онлайн' : workStatus == 'warning' ? prompt : 'Офлайн'" title="Временные неполадки на сервере. Повторите попытку позже" alt="">
      </div>
      <div id="reight_element">
        <img id="header_icon" src="../assets/icons/update.svg" alt="">
        <img id="header_icon" :src="soundOff" alt="" ref="img_sound" @click="is_sound_on">
      </div>
    </div>
</template>

<style scoped>
#header {
  display: flex;
  margin-bottom: 5px;
  align-items: center;
  justify-content: space-between;
  top: 0;
  left: 0;
  right: 0;
  padding: 8px;
}

#status {
  display: flex;
  gap: 10px;
  align-items: center;
  color: #ffff;
}

#status_circle {
  width: 16px;
  height: 16px;
  border-radius: 50%;
}

@keyframes GlitchAnimation {
  from { opacity: 1; }
  to { opacity: 0; }
}

.online {
  background-color: green;
  animation-name: GlitchAnimation;
  animation-duration: 1.6s;
  animation-iteration-count:  infinite;
}

.offline {
  background-color: rgb(151, 15, 15);
}

.warning {
  background-color: rgb(214 180 14);
  animation-name: GlitchAnimation;  
  animation-duration: 1.6s;
  animation-iteration-count:  infinite;
}

.status_img {
  width: 16px;
  height: 16px;
  margin-top: -3px;
  margin-left: -3px;
  transition: all 0.5s ease
}

.status_img:hover {
  filter: invert(1);
  transition: all 0.5s ease
}

#reight_element {
  display: flex;
  gap: 10px;
}

#header_icon {
  width: 16px;
  height: 16px;
  cursor: pointer;
}
</style>