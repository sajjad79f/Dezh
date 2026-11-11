import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],

  // vite dev (npm run dev) — پورت ۵۱۷۳ تا با سرویس dezh-web تداخل نکند
  server: {
    host: '127.0.0.1',
    port: 5173,
    strictPort: true,
    proxy: {
      '/api': {
        target: 'http://127.0.0.1:7878',
        changeOrigin: true,
      },
    },
  },

  // vite preview (سرویس dezh-web) — proxy مخصوص خودش را می‌خواند
  preview: {
    host: '0.0.0.0',
    port: 4173,
    strictPort: true,
    proxy: {
      '/api': {
        target: 'http://127.0.0.1:7878',
        changeOrigin: true,
      },
    },
  },
})
