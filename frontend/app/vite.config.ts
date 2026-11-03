import path from "node:path";
import { defineConfig, loadEnv } from "vite";
import react from "@vitejs/plugin-react";

// Cổng máy chủ Dandori mặc định (khớp `ServerConfig::default`).
const DEFAULT_BACKEND_PORT = "3100";

export default defineConfig(({ mode }) => {
  // Chỉ đọc đúng khóa `DANDORI_PORT` từ `.env` ở thư mục gốc dự án (không nạp mật khẩu hay khóa khác vào tiến trình Vite).
  const rootEnv = loadEnv(mode, path.resolve(__dirname, "../.."), "DANDORI_PORT");
  const port = process.env.DANDORI_PORT || rootEnv.DANDORI_PORT || DEFAULT_BACKEND_PORT;
  const backend = process.env.VITE_BACKEND_URL || `http://127.0.0.1:${port}`;

  return {
    plugins: [react()],
    server: {
      port: 5173,
      host: "127.0.0.1",
      // Giao diện gọi `/api/...` theo đường dẫn tương đối: chuyển tiếp sang máy chủ Dandori.
      // Máy chủ và giao diện dùng chung một cấu hình `DANDORI_PORT` (đổi ở `.env` hoặc biến môi trường).
      proxy: {
        "/api": backend,
      },
    },
  };
});
