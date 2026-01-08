<script setup>
    import { ref, onMounted, onUnmounted, defineProps } from 'vue'
    import promptImg from '../assets/icons/prompt.svg'
    
    const props = defineProps({
        workStatus: String
    })
    
    const isVisible = ref(false)
    const insideTooltip = ref({x: 0.0, y: 0.0})
    const warningMsg = 'Связь с сервером потеряна. Повторите попытку позже.'
    const workStatus = ref(props.workStatus)

    function popup() {
        isVisible.value = true
    }
    
    function getPosInsideTooltip(e) {
        insideTooltip.value = {
            x: e.clientX,
            y: e.clientY
        }
    }

    function getPosInsideMouseClick(e) {
        const x = e.clientX
        const y = e.clientY

        if (insideTooltip.value.x != x && insideTooltip.value.y != y) {
            isVisible.value = false
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
    <div @click="getPosInsideTooltip">
        <img class="status_img" :src="workStatus == 'online' ? '' : workStatus == 'warning' ? promptImg : ''" alt="" @click="popup">
        <div id="tooltip_overlay" v-show="isVisible">{{ warningMsg }}</div>
    </div>
</template>

<style>
    #tooltip_overlay {
        position: absolute;
        background-color: rgba(48, 60, 81, 0.6);
        border-radius: 8px;
        padding: 8px;
        z-index: 9999;
        box-sizing: border-box;
        border: 1px solid rgba(48, 60, 81, 0.6);
        backdrop-filter: blur(8px);
        transition: flex 0.5s;
        pointer-events: auto;
        font-size: 12px;
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
</style>