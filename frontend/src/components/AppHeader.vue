<template>
    <div id="header">
      <div id="status">
        <div id="status_cirсle"></div>
        <span>{{ workStatusOnline }}</span>
      </div>
    </div>
</template>

<script setup>
  let workStatusOnline = "Офлайн"
</script>

<style scoped>
#header {
  margin-left: 20px;
  margin-top: 20px;
}

#status {
  display: flex;
  gap: 5px;
  align-items: center;
}

#status_cirсle {
  background-color: red;
  width: 16px;
  height: 16px;
  border-radius: 50%;
}
</style>