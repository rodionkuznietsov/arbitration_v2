import { createRouter, createWebHistory } from "vue-router";

import Home from "../views/HomeView.vue";
import Chart from "../views/ChartView.vue";
import ClientKey from "../views/ClientKey.vue";
import UserHistory from "../views/UserHistory.vue";

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
    {
        path: "/my_history",
        name: "user_history",
        component: UserHistory,
    },
];

const router = createRouter({
    history: createWebHistory(),
    routes,
});

export default router;