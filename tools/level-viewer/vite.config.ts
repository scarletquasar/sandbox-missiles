import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

const apiPort = Number(process.env.LEVEL_VIEWER_API_PORT || 4173);

export default defineConfig({
  plugins: [react()],
  server: {
    proxy: {
      "/api": `http://127.0.0.1:${apiPort}`,
      "/events": {
        target: `http://127.0.0.1:${apiPort}`,
        changeOrigin: true,
      },
      "/game-assets": `http://127.0.0.1:${apiPort}`,
    },
  },
});
