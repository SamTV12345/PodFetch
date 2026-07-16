import { defineConfig } from 'vitest/config';

export default defineConfig({
    test: {
        environment: 'jsdom',
        // e2e/ contains playwright specs with their own runner (pnpm run e2e).
        exclude: ['e2e/**', 'node_modules/**'],
    },
});