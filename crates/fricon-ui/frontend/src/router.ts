import { createRouter, createWebHistory } from "vue-router";

const routes = [
  {
    path: "/",
    name: "data",
    component: () => import("./DataViewer.vue"),
  },
  {
    path: "/credits",
    name: "credits",
    component: () => import("./AppCredits.vue"),
  },
];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

export default router;
