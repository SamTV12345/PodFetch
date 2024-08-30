import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react-swc'


// https://vitejs.dev/config/
export default defineConfig({
  base:'/ui/',
  plugins: [react()],
  server:{
      host: '0.0.0.0',
    proxy:{
      '/api':{
        target: 'http://127.0.0.1:8000',
        changeOrigin: true,
        secure: false,
      },
      '/podcasts':{
        target: 'http://127.0.0.1:8000',
        changeOrigin: true,
        secure: false,
      },
      '/proxy':{
        target: 'http://127.0.0.1:8000',
        changeOrigin: true,
        secure: false,
      },
      '/manifest.json': {
        target: 'http://127.0.0.1:8000',
        changeOrigin: true,
        secure: false,
      }
    }
  }
})
