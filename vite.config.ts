import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'

export default defineConfig({
  plugins: [react(), tailwindcss()],
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
    // 排除 Rust 编译产物目录，防止 EMFILE: too many open files
    watch: {
      ignored: ['**/src-tauri/**'],
    },
  },
  // 限制依赖扫描范围，只扫描前端源码，避免扫描到 src-tauri/target 下的大量 HTML 文档
  optimizeDeps: {
    entries: ['src/**/*.{ts,tsx}'],
  },
  envPrefix: ['VITE_', 'TAURI_'],
  build: {
    target: 'esnext',
    minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
    sourcemap: !!process.env.TAURI_DEBUG,
  },
})
