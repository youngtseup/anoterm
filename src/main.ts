// 应用入口：装配 Vue + Tailwind/shadcn-vue 全局样式
import { createApp } from "vue";
import App from "./App.vue";
import "./assets/main.css";

createApp(App).mount("#app");
