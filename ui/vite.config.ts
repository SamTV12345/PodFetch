import {defineConfig, PluginOption} from 'vite'
import react from '@vitejs/plugin-react'
import {Browser} from "happy-dom";
import type {IncomingMessage} from "node:http";

const ReactCompilerConfig = {

};

function chartingLibrary(): PluginOption {
  return {
    name: 'charting-library',
    enforce: 'pre',
    apply: 'serve',
    transformIndexHtml: async (html, ctx)=>{
      const resp =  await fetch('http://localhost:8000/ui/index.html')
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
export default defineConfig({
  base:'/ui/',
  plugins: [
    chartingLibrary(),
    react({
      babel: {
        plugins: [["babel-plugin-react-compiler", ReactCompilerConfig]],
      },
    }),
  ],
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
})
