<script setup>
  import { defineExpose, ref } from 'vue';

  const workStatus = ref(false)

  function change_work_status(status) {
    workStatus.value = status
  }

  defineExpose({ change_work_status })
</script>

<template>
    <div id="header">
      <div id="status">
        <div id="status_circle" :class="workStatus ? 'online' : 'offline'"></div>
        <span>{{ workStatus ? 'Онлайн' : 'Офлайн' }}</span>
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
  color: #ffff;
}

#status_circle {
  width: 16px;
  height: 16px;
  border-radius: 50%;
}

.online {
  background-color: green;
}

.offline {
  background-color: rgb(151, 15, 15);
}

#reight_element {
  display: flex;
  gap: 20px;
}

#header_icon {
  width: 16px;
  height: 16px;
}
</style>