<script setup>
  import { ref } from 'vue';
  import soundOn from '../assets/icons/sound_on.svg'
  import soundOff from '../assets/icons/sound_off.svg'
  import Tooltip from './TooltipWidget.vue';
import { useUserState } from '@/stores/user_state';

  const userState = useUserState()

  const img_sound = ref(null)
  const isOn = ref(false)

  function is_sound_on() {
    if (img_sound.value) {
      isOn.value = !isOn.value
      img_sound.value.src = isOn.value ? soundOn : soundOff
    }
  }
</script>

<template>
    <div id="header">
      <div id="status">
        <div id="status_circle" :class="userState.currentStatus == 'online' ? 'online' : userState.currentStatus == 'warning' ? 'warning' : 'offline'"></div>
        <span>{{ userState.currentStatus == 'online' ? 'Онлайн' : userState.currentStatus == 'warning' ? 'Неполадки' : 'Офлайн' }}</span>
        <div v-if="userState.currentStatus == 'warning'">
          <Tooltip/>
        </div>
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
    padding: var(--default-padding);
  }
  
  #status {
    display: flex;
    gap: 10px;
    align-items: center;
  }
  
  #status_circle {
    width: var(--default-icon-size);
    height: var(--default-icon-size);
    border-radius: 50%;
  }

  @keyframes GlitchAnimation {
    from { opacity: 1; }
    to { opacity: 0; }
  }

  .online {
    background-color: var(--color-success);
    animation-name: GlitchAnimation;
    animation-duration: 1.6s;
    animation-iteration-count:  infinite;
  }

  .offline {
    background-color: var(--color-error);
  }

  .warning {
    background-color: rgb(214 180 14);
    animation-name: GlitchAnimation;  
    animation-duration: 1.6s;
    animation-iteration-count:  infinite;
  }

  #reight_element {
    display: flex;
    gap: 10px;
  }

  #header_icon {
    width: var(--default-icon-size);
    height: var(--default-icon-size);
    cursor: pointer;
  } 
</style>