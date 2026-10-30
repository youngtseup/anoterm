// 应用入口：装配 Vue + Tailwind/shadcn-vue 全局样式
import { createApp } from "vue";
import App from "./App.vue";
import "./assets/main.css";

// 桌面客户端不显示 WebView 自带的浏览器右键菜单（应用内自定义菜单自行 preventDefault）
window.addEventListener("contextmenu", (e) => e.preventDefault());

createApp(App).mount("#app");
