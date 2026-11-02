import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  server: {
    port: 5173,
    host: "127.0.0.1",
    // Giao diện gọi `/api/...` theo đường dẫn tương đối: chuyển tiếp sang máy chủ Dandori (mặc định cổng 3000).
    proxy: {
      "/api": "http://127.0.0.1:3000",
    },
  },
});
