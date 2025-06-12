import { createApp } from "vue";
import "./style.css";
import PrimeVue from "primevue/config";
import Aura from "@primeuix/themes/aura";
import App from "./App.vue";

createApp(App)
  .use(PrimeVue, { theme: { preset: Aura } })
  .mount("#app");
