import { createRouter, createWebHistory } from "vue-router";

import Home from "../views/HomeView.vue";
import Chart from "../views/ChartView.vue";

const routes = [
    {
        path: "/",
        component: Home,
    },
    {
        path: "/chart",
        name: "chart",
        component: Chart,
    },
];

const router = createRouter({
    history: createWebHistory(),
    routes,
});

export default router;