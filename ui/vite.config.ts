import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react-swc'
import {VitePWA, VitePWAOptions} from "vite-plugin-pwa";


const pwaConfig: Partial<VitePWAOptions> = {
    registerType: 'autoUpdate',
    devOptions: {
        enabled: true,
        type: 'module'
    },
    scope: '/ui/',
    manifest: {
        name: 'PodFetch',
        short_name: 'PodFetch',
        description: 'A sleek and efficient podcast player',
        theme_color: '#000000',
        background_color: '#ffffff',

        display: 'standalone',
        start_url: '/ui/',
        icons: [
        {
            src: '/ui/pwa-192x192.png',
            sizes: '192x192',
            type: 'image/png',
        },
        {
            src: '/ui/pwa-512x512.png',
            sizes: '512x512',
            type: 'image/png',
        },
        {
            src: '/ui/maskeable-icon-512x512.png',
            sizes: '512x512',
            type: 'image/png',
            purpose: 'maskable',
        },
        ],
    }
}


// https://vitejs.dev/config/
export default defineConfig({
  base:'/ui/',
  plugins: [react(), VitePWA(pwaConfig)],
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
      }
    }
  }
})
