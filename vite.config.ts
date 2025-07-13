import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// https://vitejs.dev/config/
export default defineConfig(async ({ mode }) => {
  const config = {
    plugins: [react()],
    define: {
      __IS_TAURI__: mode === 'tauri',
    },
  };

  if (mode === 'tauri') {
    // Tauri-specific configuration
    return {
      ...config,
      // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
      clearScreen: false,
      server: {
        port: 1420,
        strictPort: true,
        watch: {
          ignored: ["**/src-tauri/**"],
        },
      },
    };
  } else {
    // Web-specific configuration
    return {
      ...config,
      server: {
        port: 3000,
        host: true,
        proxy: {
          // Proxy API calls to the Rust backend in development
          '/api': {
            target: 'http://localhost:3001',
            changeOrigin: true,
            secure: false,
          },
        },
      },
      build: {
        outDir: 'dist',
        rollupOptions: {
          external: ['@tauri-apps/api/core'],
          output: {
            globals: {
              '@tauri-apps/api/core': 'window.__TAURI_API__',
            },
          },
        },
      },
      // Environment variables
      define: {
        ...config.define,
        __API_URL__: JSON.stringify(
          process.env.VITE_API_URL || 
          (process.env.NODE_ENV === 'production' ? '' : 'http://localhost:3000')
        ),
      },
    };
  }
});