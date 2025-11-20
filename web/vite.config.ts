import { defineConfig } from "vite";
import { reactRouter } from "@react-router/dev/vite";
import { ViteImageOptimizer } from "vite-plugin-image-optimizer";

// https://vite.dev/config/
export default defineConfig({
  server: {
    port: 5173,
    host: true,
    allowedHosts: ["dev.dream-house.kr"],
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
