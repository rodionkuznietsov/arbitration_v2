import { createRouter, createWebHistory } from "vue-router";

import Home from "../views/HomeView.vue";
import Chart from "../views/ChartView.vue";
import ClientKey from "../views/ClientKey.vue";
import UserHistory from "../views/UserHistory.vue";
import LatestBestSpread from "@/views/LatestBestSpread.vue";

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
    {
        path: "/latest_best_spreads",
        name: "latest_best_spreads",
        component: LatestBestSpread
    }
];

const router = createRouter({
    history: createWebHistory(),
    routes,
});

export default router;