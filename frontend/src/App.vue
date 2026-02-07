<template>
  <div id="app">
    <router-view />

    <footer id="footer">
      <AppMenu />
    </footer>
  </div>
</template>

<script setup>
import { onMounted } from 'vue'
import WebApp from "@twa-dev/sdk"
import AppMenu from './components/AppMenu.vue';

onMounted(() => {
  WebApp.ready()
  WebApp.expand()
})

</script>

<style>
  @import url('https://fonts.googleapis.com/css2?family=PT+Serif:ital,wght@0,400;0,700;1,400;1,700&family=Vollkorn+SC:wght@400;600;700;900&display=swap');

  :root {
    --basic-Bg: #202630;
    --color-input-border: #b7bcc4;
    --color-popup: #333B47;
    --color-success: #2EBD85;
    --color-success-opacity-0_15: hsl(157 61% 46% / 0.15);
    --color-error: #F6465D;
    --color-error-opacity-0_15: hsl(352 91% 62% / 0.15);
    --default-font-color: #EAECEF;
    --default-font-size: 16px;
    --default-padding: 8px;
    --default-icon-size: 16px;
    --default-border-radius: 8px;
    --default-input-color: #303c51;
  }
 
  #form {
    display: grid;
    gap: 10px;
    width: 100%;
    box-sizing: border-box;
    justify-content: space-around;
    grid-template-columns: repeat(auto-fit, minmax(100px, 1fr));
    border-radius: var(--default-border-radius);
    padding: var(--default-padding);
    font-size: var(--default-font-size);
  }

  #form_label-with_icon {
    display: flex;
    flex-direction: row;
    align-items: center;
    justify-content: flex-start;
    gap: 5px;
  }

  #form_label-with_icon img {
    width: var(--default-icon-size);
    height: var(--default-icon-size);
  }

  .img_reverse {
    transform: rotate(60deg);
  }

  #form-column{
    display: flex;
    flex-direction: column;
  }

  .form_input {
    width: 100%;
    padding: var(--default-padding);
    border: none;
    color: var(--default-font-color);
    border-radius: var(--default-border-radius);
    font-size: var(--default-font-size);
    margin-top: 10px;
    background-color: #303c5199;
    outline: none;
    box-sizing: border-box;
    text-transform: uppercase;
  }

  .form-group {
    display: flex;
    flex-direction: column;
  }

  #footer {
    display: flex;
    flex-direction: column;
    margin-top: 10px;
    padding: var(--default-padding);
    overflow-y: visible;
    margin-bottom: 70px;
  }

  #run_buttons {
    display: flex;
    gap: 10px;
  }

  #start, #stop {
    flex: 1;
    padding: var(--default-padding);
    border: none;
    color: var(--default-font-color);
    border-radius: var(--default-border-radius);
    font-size: var(--default-font-size);
    transition: all 0.25s;
  }

  #start:hover, #stop:hover {
    filter: opacity(75%);
    cursor: pointer;
    transition: all 0.25s;
  }

  #start {
    background-color: var(--color-success);
  } 

  #stop {
    background-color: var(--color-error);
  } 

  #app {
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
    text-align: left;
    color: var(--default-font-color);
    position: relative;
    font-family: "PT Serif", serif;
    font-weight: 400;
    font-style: normal;
  }

  body {
    background-color: var(--basic-Bg);
    margin-bottom: 10px;
  }
</style>
