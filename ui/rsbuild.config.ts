import { defineConfig } from '@rsbuild/core';
import { pluginReact } from '@rsbuild/plugin-react';
import { pluginBabel } from '@rsbuild/plugin-babel';
import { pluginSass } from '@rsbuild/plugin-sass';

const ReactCompilerConfig = {
    // ReactCompilerConfig hier einfügen, falls benötigt
};

export default defineConfig({
    plugins: [pluginReact(),
        pluginBabel({
            include: /\.(?:jsx|tsx)$/,
            babelLoaderOptions(opts) {
                opts.plugins?.unshift([
                    'babel-plugin-react-compiler',
                    ReactCompilerConfig,
                ]);
            },
        }),
        pluginSass(),
    ],
    source: {
      entry: {
        main: './src/main.tsx'
      }
    },
    html: {
      template: './index.html',
    },
    server: {
        base: '/ui/',
      host: '0.0.0.0',
        proxy: {
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
    },
    output: {
        distPath: {
            root: './dist'
        }
    }
});
