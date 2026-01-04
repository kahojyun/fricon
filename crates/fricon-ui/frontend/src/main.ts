import { createApp } from "vue";
import "./style.css";
import "primeicons/primeicons.css";
import PrimeVue from "primevue/config";
import Aura from "@primeuix/themes/aura";
import App from "./App.vue";
import router from "./router.ts";

createApp(App)
  .use(PrimeVue, { theme: { preset: Aura } })
  .use(router)
  .mount("#app");
