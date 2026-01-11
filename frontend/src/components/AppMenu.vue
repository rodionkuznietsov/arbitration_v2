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
        background-color: rgba(48, 60, 81, 0.404);
        left: 0;
        right: 0;
        margin-left: 60px;
        margin-right: 60px;
        border-radius: 30px;
        font-size: 16px;
    }

    #menu_overlay {
        display: flex;
        gap: 50px;
        background-color: #303c5127;
        position: relative;
        backdrop-filter: blur(8px); 
        -webkit-backdrop-filter: blur(8px);
        width: 100%;
        height: 100%;
        padding: 8px;
        box-sizing: border-box;
        justify-content: center;
        border-radius: 30px;
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