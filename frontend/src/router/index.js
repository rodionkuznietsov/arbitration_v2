import { createRouter, createWebHistory } from "vue-router";

import Home from "../views/HomeView.vue";
import Chart from "../views/ChartView.vue";
import ClientKey from "../views/ClientKey.vue";

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
    {
        path: "/client_keys",
        name: "client_keys",
        component: ClientKey,
    },
];

const router = createRouter({
    history: createWebHistory(),
    routes,
});

export default router;