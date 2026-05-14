import {defineConfig, PluginOption} from 'vite'
import react from '@vitejs/plugin-react'
import {Browser} from "happy-dom";
import type {IncomingMessage} from "node:http";
import path from "node:path";
import {fileURLToPath} from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

const ReactCompilerConfig = {

};

function chartingLibrary(): PluginOption {
  return {
    name: 'charting-library',
    enforce: 'pre',
    apply: 'serve',
    transformIndexHtml: async (html, ctx)=>{
      // Tell the backend which host the browser actually talks to, so its
      // embedded `data-config.serverUrl` reflects the Vite dev server rather
      // than the backend's internal bind address. Vite's bind host may be
      // `0.0.0.0` or `true`; the browser reaches it via `localhost`.
      const port = ctx.server?.config?.server?.port ?? 5173
      const host = `localhost:${port}`
      const resp = await fetch('http://localhost:8000/ui/index.html', {
        headers: {
          'x-forwarded-host': host,
          'x-forwarded-proto': 'http',
        },
      })
      const htmlFromServer = await resp.text()
      const browser = new Browser()
      const page = browser.newPage();

      page.url = 'https://example.com';
      page.content = htmlFromServer;
      const configFromServer = page.mainFrame.document.getElementById('config').getAttribute('data-config')
      const page2 = browser.newPage();

      page2.url = 'https://example.com/new';
      page2.content = html;
      let configNode = page2.mainFrame.document.createElement('span', { is: 'span' });
      configNode.id = 'config'
      configNode.setAttribute('data-config', configFromServer)
      page2.mainFrame.document.getElementsByTagName('body')[0].appendChild(configNode)

      const servableHtml = page2.mainFrame.content
      await browser.close()

      return servableHtml
    }
  };
}

function forwardRequestOrigin(proxyReq: any, req: IncomingMessage) {
  const host = req.headers.host;
  if (host) {
    proxyReq.setHeader('x-forwarded-host', host);
    proxyReq.setHeader('x-forwarded-proto', 'http');
  }
}

function createBackendProxy(ws = false) {
  return {
    target: 'http://127.0.0.1:8000',
    changeOrigin: true,
    secure: false,
    ws,
    configure: (proxy: any) => {
      proxy.on('proxyReq', (proxyReq: any, req: IncomingMessage) => {
        forwardRequestOrigin(proxyReq, req);
      });
      proxy.on('proxyReqWs', (proxyReq: any, req: IncomingMessage) => {
        forwardRequestOrigin(proxyReq, req);
      });
    }
  }
}

// https://vitejs.dev/config/
export default defineConfig(({command}) => ({
  base:'/ui/',
  plugins: [
    chartingLibrary(),
    react({
      // The React Compiler walks every component for auto-memoization
      // (68% of plugin time on prod builds per PLUGIN_TIMINGS). HMR and
      // React DevTools don't need that analysis - skip it in `vite dev`.
      babel: command === 'build'
        ? {plugins: [["babel-plugin-react-compiler", ReactCompilerConfig]]}
        : undefined,
    }),
  ],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  build: {
    // Default 500 kB is conservative for an SPA with ~30 routes. Our
    // entry chunk is ~528 kB / 165 kB gzipped which is healthy.
    chunkSizeWarningLimit: 700,
  },
  server:{
      host: '0.0.0.0',
    proxy:{
      '/api': createBackendProxy(),
      '/socket.io': createBackendProxy(true),
      '/podcasts': createBackendProxy(),
      '/proxy': createBackendProxy(),
      '/rss': createBackendProxy(),
      '/manifest.json': createBackendProxy(),
    }
  },
}))
