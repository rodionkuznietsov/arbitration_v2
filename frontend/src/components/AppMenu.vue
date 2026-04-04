<script setup>
    const elements = [
        { name: 'График', icon: '../assets/icons/graph.svg', link: '/chart' },
        { name: 'Курс', icon: '../assets/icons/growth.svg', link: '/' },
        { name: 'История', icon: '../assets/icons/history.svg', link: '/my_history' },
        { name: 'Топ спреды', icon: '../assets/icons/latest_best_spreads.svg', link: '/latest_best_spreads' },
        { name: 'Ключи', icon: '../assets/icons/key.svg', link: '/client_keys' }
    ]
</script>

<template>
    <div id="app_menu">
        <div class="app_menu_wrapper">
            <div id="menu_elements">
                <div class="menu-item" id="menu_element" v-for="(element, index) in elements" :key="index">
                    <router-link class="router-link" :to="element.link" active-class="router-link-active">
                        <img class="icon_svg" :src="element.icon" alt="">
                        <span class="router-link-text">{{ element.name }}</span>
                    </router-link>
                </div>
                <!-- <div id="menu_element">
                    <router-link class="router-link" to="/chart">
                        <img src="../assets/icons/graph.svg" alt="">
                        <span class="router-link-text">График</span>
                    </router-link>
                </div>
                <div id="menu_element">
                    <router-link class="router-link" active-class="router-link-active" to="/my_history">
                        <img src="../assets/icons/history.svg" alt="">
                        <span class="router-link-text">История</span>
                    </router-link>
                </div>
                <div id="menu_element">
                    <router-link class="router-link" to="/">
                        <img src="../assets/icons/growth.svg" alt="">
                        <span class="router-link-text">Курс</span>
                    </router-link>
                </div>
                <div id="menu_element">
                    <router-link class="router-link" active-class="router-link-active" to="/latest_best_spreads">
                        <img src="../assets/icons/latest_best_spreads.svg" alt="">
                        <span class="router-link-text">Топ спреды</span>
                    </router-link>
                </div> 
                <div id="menu_element">
                    <router-link class="router-link" active-class="router-link-active" to="/client_keys">
                        <img src="../assets/icons/key.svg" alt="">
                        <span class="router-link-text">Ключи</span>
                    </router-link>
                </div> -->
            </div>  
        </div>
    </div>
</template>

<style scoped>
    #app_menu {
        display: flex;
        position: fixed;
        bottom: var(--default-bottom-px);
        background-color: var(--basic-Bg);
        z-index: 10;
        right: var(--wrapper-margin-right-left-px);
        left: var(--wrapper-margin-right-left-px);
    }

    .app_menu_wrapper {
        display: flex;
        justify-content: center;
        width: 100%;
        background-color: #00000013;
        border-radius: 15px;
        flex-direction: column;
        align-items: center;
    }

    .router-link {
        color: var(--default-font-color);
        text-decoration: none;
        display: flex;
        flex-direction: row;
        align-items: center;
        border-radius: 30px;
        padding: var(--default-padding);
        font-size: var(--router-link-font-size);
    }

    .router-link-active {
        background-color: var(--color-success);
        font-weight: 600;
        transition: background-color 0.5s ease;
    }

    .router-link-text {
        width: min-content;
        text-align: center;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    #menu_elements {
        display: grid;
        grid-template-columns: repeat(3, 1fr);
        grid-template-rows: repeat(3, auto);
        position: relative;
        backdrop-filter: blur(8px);
        -webkit-backdrop-filter: blur(8px);
        width: 100%;
        height: 100%;
        padding: var(--default-padding);
        box-sizing: content-box;
        border-radius: 15px;
    }

    #menu_elements .menu-item:nth-child(2) {
        grid-column: 2;
        grid-row: 2;
    }

    #menu_elements .menu-item:nth-child(3) {
        grid-column: 3;
    }

    #menu_elements .menu-item:nth-child(4) {
        grid-column: 1;
        grid-row: 3;
    }

    #menu_elements .menu-item:nth-child(5) {
        grid-column: 3;
        grid-row: 3;
    }

    #menu_element {
        display: flex;
        flex-direction: column;
        justify-content: center;
        align-items: center;
        padding: var(--default-padding);
    }

    #menu_element, img {
        height: var(--default-icon-size);
        margin-right: 5px;
    }

    #menu_element:hover {
        filter: opacity(0.7);
        border-radius: var(--default-border-radius);
        cursor: pointer;
    }
</style>