import { defineConfig } from '@playwright/test'

/**
 * Full-stack e2e suite for the transcript features: playwright owns the
 * lifecycle of the fixture feed server, the mock whisper server and the real
 * podfetch binary (fresh sqlite DB per run, serving the built UI).
 *
 * Prerequisites (both are verified by start-podfetch.sh):
 *   cargo build --no-default-features --features sqlite
 *   cd ui && pnpm run build
 */
export default defineConfig({
    testDir: './e2e/tests',
    globalSetup: './e2e/global-setup.ts',
    timeout: 90_000,
    expect: { timeout: 10_000 },
    retries: process.env.CI ? 1 : 0,
    // All tests share one server + seeded podcast; keep execution deterministic.
    workers: 1,
    reporter: process.env.CI ? [['github'], ['html', { open: 'never' }]] : 'list',
    use: {
        baseURL: 'http://127.0.0.1:8000',
        locale: 'en-US',
        trace: 'retain-on-failure',
        screenshot: 'only-on-failure',
    },
    webServer: [
        {
            command: 'node e2e/servers/fixture-feed.mjs',
            url: 'http://127.0.0.1:9123/feed.xml',
            reuseExistingServer: false,
            timeout: 15_000,
        },
        {
            command: 'node e2e/servers/mock-whisper.mjs',
            url: 'http://127.0.0.1:9998/health',
            reuseExistingServer: false,
            timeout: 15_000,
        },
        {
            command: 'bash e2e/servers/start-podfetch.sh',
            url: 'http://127.0.0.1:8000/api/v1/sys/config',
            reuseExistingServer: false,
            timeout: 120_000,
            stdout: 'pipe',
            stderr: 'pipe',
        },
    ],
})
