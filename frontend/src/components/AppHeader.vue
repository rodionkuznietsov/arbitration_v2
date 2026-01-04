<script setup>
  import { defineProps, ref } from 'vue';

  const props = defineProps({
    online: Boolean
  })

  const workStatus = ref(props.online)
  workStatus.value = true
</script>

<template>
    <div id="header">
      <div id="status">
        <div id="status_circle" :class="workStatus ? 'online' : 'offline'"></div>
        <span>{{ workStatus ? 'Online' : 'Offline' }}</span>
      </div>
    </div>
</template>

<style scoped>
#header {
  margin-left: 20px;
  margin-top: 20px;
  align-items: center;
}

#status {
  display: flex;
  gap: 5px;
  align-items: center;
}

#status_circle {
  width: 20px;
  height: 20px;
  border-radius: 50%;
}

.online {
  background-color: green;
}

.offline {
  background-color: red;
}
</style>