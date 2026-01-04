<script setup>
  import { defineProps, ref } from 'vue';

  const props = defineProps({
    online: Boolean
  })

  const workStatus = ref(props.online)
  workStatus.value = false
</script>

<template>
    <div id="header">
      <div id="status">
        <div id="status_circle" :class="workStatus ? 'online' : 'offline'"></div>
        <span>{{ workStatus ? 'Online' : 'Offline' }}</span>
      </div>
      <div id="reight_element">
        <img id="header_icon" src="../assets/icons/update.svg" alt="">
        <img id="header_icon" src="../assets/icons/sound_off.svg" alt="">
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
  gap: 20px;
  align-items: center;
}

#status_circle {
  width: 25px;
  height: 25px;
  border-radius: 50%;
}

.online {
  background-color: green;
}

.offline {
  background-color: red;
}

#reight_element {
  display: flex;
  gap: 20px;
}

#header_icon {
  width: 25px;
  height: 25px;
}
</style>