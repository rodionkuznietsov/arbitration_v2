<script setup>

</script>

<template>
    <div id="app_menu">
        <div id="menu_overlay">
            <div id="menu_element">
                <img src="../assets/icons/growth.svg" alt="">
                <span>Курс</span>
            </div>
            <div id="menu_element">
                <img src="../assets/icons/logs.svg" alt="">
                <span>История</span>
            </div>
        </div>  
    </div>
</template>

<style scoped>
    #app_menu {
        display: flex;
        position: fixed;
        bottom: 10px;
        background-color: rgba(255, 255, 255, 0.05);
        box-sizing: border-box;
        left: 0;
        right: 0;
        margin-left: 50px;
        margin-right: 50px;
        border-radius: 8px;
        font-size: 16px;
    }

    #menu_overlay {
        display: flex;
        gap: 50px;
        background-color: rgba(255, 255, 255, 0.02);
        position: relative;
        backdrop-filter: blur(20px); 
        -webkit-backdrop-filter: blur(20px);
        width: 100%;
        height: 100%;
        padding: 8px;
        border-radius: 8px;
        box-sizing: border-box;
        justify-content: center;
    }

    #menu_element {
        display: flex;
        flex-direction: column;
        justify-content: center;
        align-items: center;
        padding: 8px;
    }

    #menu_element, img {
        width: 16px;    
        height: 16px;
    }

    #menu_element:hover {
        filter: opacity(0.7);
        border-radius: 8px;
        cursor: pointer;
    }
</style>