import "@/assets/styles/tailwind.css";

import { createApp } from "vue";

import App from "@/App.vue";

document.documentElement.dataset.theme = "dark";
createApp(App).mount("#app");
