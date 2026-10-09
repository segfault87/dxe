import { defineConfig } from "vite";
import { reactRouter } from "@react-router/dev/vite";
import { ViteImageOptimizer } from "vite-plugin-image-optimizer";

// https://vite.dev/config/
export default defineConfig({
  server: {
    port: 5173,
    host: true,
    allowedHosts: ["dev.dream-house.kr"],
    proxy: {
      "/api": {
        target: "http://localhost:8000",
        changeOrigin: true,
      },
    },
  },
  plugins: [
    reactRouter(),
    ViteImageOptimizer({
      jpeg: {
        quality: 90,
      },
    }),
  ],
});
