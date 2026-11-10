import { expect, test, type APIRequestContext, type Page } from '@playwright/test'
import { FEED_BASE, episodeRow } from '../helpers'

// Library maintenance flows (issues #2306-#2312): podcast cover from
// <itunes:image> under the configured cover filename, the download toast,
// marking all episodes (un)played, a persistent sort order, clearing large
// numbers of notifications and the explained naming settings.
//
// Uses its own podcast (fixture /feed3.xml) so the seeded podcast other suites
// rely on is left untouched; settings are restored afterwards.

const BASE_URL = 'http://127.0.0.1:8000'
const EPISODE_COUNT = 7

let api: APIRequestContext
let podcastId: string
let originalSettings: Record<string, unknown>

test.beforeAll(async ({ playwright }) => {
    api = await playwright.request.newContext({ baseURL: BASE_URL })

    originalSettings = await (await api.get('/api/v1/settings')).json()
    const settingsResponse = await api.put('/api/v1/settings', {
        data: { ...originalSettings, coverFilename: 'cover' },
    })
    expect(settingsResponse.ok()).toBeTruthy()

    const addResponse = await api.post('/api/v1/podcasts/feed', {
        data: { rssFeedUrl: `${FEED_BASE}/feed3.xml` },
    })
    expect(addResponse.ok(), await addResponse.text()).toBeTruthy()
    podcastId = (await addResponse.json()).id

    // Wait until the episodes are inserted.
    await expect
        .poll(
            async () =>
                (await (await api.get(`/api/v1/podcasts/${podcastId}/episodes?only_unlistened=false`)).json())
                    .length,
            { timeout: 30_000 }
        )
        .toBe(EPISODE_COUNT)
})

test.afterAll(async () => {
    await api.delete(`/api/v1/podcasts/${podcastId}`, { data: { delete_files: true } })
    await api.put('/api/v1/settings', { data: originalSettings })
    await api.dispose()
})

const detailPage = () => `/ui/podcasts/${podcastId}/episodes`

test('a feed with only <itunes:image> gets its cover, saved under the cover filename', async () => {
    const podcast = await (await api.get(`/api/v1/podcasts/${podcastId}`)).json()

    expect(podcast.original_image_url).toBe(`${FEED_BASE}/itunes-cover.png`)
    expect(podcast.image_url).toMatch(/\/cover\.png$/)

    const image = await api.get(podcast.image_url)
    expect(image.ok()).toBeTruthy()
    expect(image.headers()['content-type']).toContain('image/png')
})

test('the download button says the download started, then that it finished', async ({ page }) => {
    await page.goto(detailPage())

    // The oldest episode is outside the initial prefill and not downloaded.
    await episodeRow(page, 'iTunes Episode 1').locator('svg.lucide-cloud-download').click()

    await expect(page.getByText('Download to server started', { exact: true })).toBeVisible()
    await expect(page.getByText('Episode downloaded to server', { exact: true })).toBeVisible({
        timeout: 30_000,
    })
})

const setUnplayedFilter = async (page: Page, enabled: boolean) => {
    const toggle = page.locator('div.flex.gap-4.justify-end', { hasText: 'Unplayed' })
    const input = toggle.locator('input[type="checkbox"]')
    if ((await input.isChecked()) !== enabled) {
        await toggle.locator('div.cursor-pointer').first().click()
    }
    await expect(input).toBeChecked({ checked: enabled })
}

test('all episodes can be marked as played and unplayed', async ({ page }) => {
    await page.goto(detailPage())
    await setUnplayedFilter(page, true)
    await expect(page.locator('div[id^="episode_"]')).toHaveCount(EPISODE_COUNT)

    await page.getByRole('button', { name: 'Mark all as played' }).click()
    await page.getByRole('dialog').getByRole('button', { name: 'Mark all as played' }).click()
    await expect(page.getByText(`${EPISODE_COUNT} episodes marked as played`)).toBeVisible()
    await expect(page.locator('div[id^="episode_"]')).toHaveCount(0)

    await page.getByRole('button', { name: 'Mark all as unplayed' }).click()
    await page.getByRole('dialog').getByRole('button', { name: 'Mark all as unplayed' }).click()
    await expect(page.getByText(`${EPISODE_COUNT} episodes marked as unplayed`)).toBeVisible()
    await expect(page.locator('div[id^="episode_"]')).toHaveCount(EPISODE_COUNT)
})

const sortSelect = (page: Page) =>
    page.locator('[data-slot="select-trigger"]', { has: page.locator('svg.lucide-arrow-up-down') })

const chooseSortOrder = async (page: Page, label: string) => {
    await sortSelect(page).click()
    await page.getByRole('option', { name: label, exact: true }).click()
    await expect(sortSelect(page)).toContainText(label)
}

test('the podcast sort order survives reloads', async ({ page }) => {
    await page.goto('/ui/podcasts')

    for (const order of ['A-Z', 'Z-A', 'A-Z']) {
        await chooseSortOrder(page, order)
        // The stored filter must not be overwritten while the page loads.
        for (let i = 0; i < 2; i++) {
            await page.reload()
            await expect(sortSelect(page)).toContainText(order)
        }
    }
})

test('clear all dismisses thousands of notifications with one request', async ({ page }) => {
    const total = 1500
    let notifications = Array.from({ length: total }, (_, index) => ({
        id: `00000000-0000-4000-8000-${String(index).padStart(12, '0')}`,
        typeOfMessage: 'Download',
        message: `Download failed ${index}`,
        createdAt: new Date(Date.UTC(2026, 10, 1, 0, 0, index)).toISOString(),
        status: 'unread',
    }))
    await page.route('**/api/v1/notifications/unread', route => route.fulfill({ json: notifications }))

    const dismissRequests: string[] = []
    page.on('request', request => {
        const url = new URL(request.url())
        if (url.pathname.startsWith('/api/v1/notifications/dismiss')) {
            dismissRequests.push(url.pathname)
        }
    })
    await page.route('**/api/v1/notifications/dismiss-all', route => {
        notifications = []
        return route.fulfill({ status: 200 })
    })

    await page.goto('/ui/podcasts')
    await page.getByRole('button', { name: 'Open notifications' }).click()

    // Only a bounded number of entries is rendered.
    await expect(page.getByRole('button', { name: 'Dismiss notification' })).toHaveCount(200)
    await expect(page.getByText(`+ ${total - 200} more`)).toBeVisible()

    await page.getByRole('button', { name: 'Clear all' }).click()
    await expect(page.getByText('No notifications').first()).toBeVisible()
    expect(dismissRequests).toEqual(['/api/v1/notifications/dismiss-all'])
})

test('the naming settings explain what "No subfolder per episode" does', async ({ page }) => {
    await page.goto('/ui/settings/naming')

    const legend = page.locator('legend', { hasText: 'No subfolder per episode' })
    await legend.getByRole('button').click()

    const dialog = page.getByRole('dialog')
    await expect(dialog).toContainText('stored directly in the podcast')
    await expect(dialog).toContainText('its own subfolder')
})
