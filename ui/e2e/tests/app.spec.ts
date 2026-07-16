import { expect, test } from '@playwright/test'
import {
    FEED_BASE,
    audioCurrentTime,
    openDetailedPlayer,
    playEpisode,
    readSeed,
} from '../helpers'

const seed = readSeed()
const detailPage = `/ui/podcasts/${seed.podcastId}/episodes`

test('podcasts page lists the fixture podcast and opens its detail view', async ({ page }) => {
    await page.goto('/ui/podcasts')

    const card = page.getByText('Transcript E2E Podcast', { exact: true })
    await expect(card).toBeVisible()
    await card.click()

    await expect(page).toHaveURL(new RegExp(`/ui/podcasts/${seed.podcastId}/episodes`))
    await expect(page.getByText('Smoke Episode 1', { exact: true })).toBeVisible()
    await expect(page.getByText('Plain Episode 2', { exact: true })).toBeVisible()
})

test('a podcast can be added through the rss feed dialog', async ({ page }) => {
    await page.goto('/ui/podcasts')

    await page.getByRole('button', { name: 'Add new' }).click()
    await page.getByText('RSS feed URL', { exact: true }).click()
    await page.getByPlaceholder('RSS feed URL').fill(`${FEED_BASE}/feed2.xml`)
    await page.getByRole('button', { name: 'Add', exact: true }).click()

    // Fresh navigation re-fetches the podcast list from the server.
    await expect(async () => {
        await page.goto('/ui/podcasts')
        await expect(page.getByText('Second Fixture Podcast', { exact: true })).toBeVisible({
            timeout: 2_000,
        })
    }).toPass({ timeout: 30_000 })
})

test('a podcast can be deleted from the manage-podcasts settings', async ({ page }) => {
    await page.goto('/ui/settings/podcasts')

    const nameCell = page.locator('span', { hasText: 'Second Fixture Podcast' })
    await expect(nameCell).toBeVisible()
    // Row layout is [checkbox root][hidden input][name span] — the clickable
    // checkbox root is two siblings back.
    await nameCell.locator('xpath=preceding-sibling::*[2]').click()

    await page.getByRole('button', { name: 'Delete podcast only' }).click()
    await page.getByRole('button', { name: 'Delete podcast', exact: true }).click()

    await expect(page.locator('span', { hasText: 'Second Fixture Podcast' })).not.toBeVisible()

    // Gone from the podcasts overview too.
    await page.goto('/ui/podcasts')
    await expect(page.getByText('Transcript E2E Podcast', { exact: true })).toBeVisible()
    await expect(page.getByText('Second Fixture Podcast', { exact: true })).not.toBeVisible()
})

test('playing an episode drives the bottom player bar', async ({ page }) => {
    await page.goto(detailPage)
    await playEpisode(page, 'Smoke Episode 1')

    // Bottom bar shows the running episode and time advances.
    await expect(page.getByTestId('audio-player-bar').getByText('Smoke Episode 1')).toBeVisible()
    const t1 = await audioCurrentTime(page)
    await expect.poll(() => audioCurrentTime(page)).toBeGreaterThan(t1)

    // Pause stops the clock, play resumes it. The play/pause control is a
    // span wrapping the icon, and it exists twice (bottom bar + the hidden
    // detailed player portal), so filter for the visible one.
    await page.locator('svg.lucide-pause').locator('..').filter({ visible: true }).click()
    const paused = await audioCurrentTime(page)
    await page.waitForTimeout(700)
    expect(await audioCurrentTime(page)).toBeCloseTo(paused, 1)

    await page.locator('svg.lucide-play').locator('..').filter({ visible: true }).click()
    await expect.poll(() => audioCurrentTime(page)).toBeGreaterThan(paused)
})

test('detailed player shows description and chapters tabs', async ({ page }) => {
    await page.goto(detailPage)
    await playEpisode(page, 'Smoke Episode 1')
    await openDetailedPlayer(page)

    // Description tab is active by default and renders the episode description
    // (the detailed player portal is appended to <body>, hence .last()).
    await expect(page.getByText('Fixture episode Smoke Episode 1').last()).toBeVisible()

    // Chapters tab is reachable (the fixture has none, so just the header row).
    await page.getByText('Chapters', { exact: true }).click()
    await expect(page.getByText('Description', { exact: true })).toBeVisible()
})

test('episode metadata search finds fixture episodes', async ({ page }) => {
    await page.goto('/ui/podcasts/search')

    await page.locator('#search-input').fill('Plain Episode')
    await expect(page.getByText('Plain Episode 2', { exact: true })).toBeVisible()
    await expect(page.getByText('Fixture episode Plain Episode 2', { exact: true })).toBeVisible()
})

test('podcast settings modal exposes the per-podcast configuration', async ({ page }) => {
    await page.goto(detailPage)

    await page.locator('svg.lucide-settings').locator('xpath=ancestor::button[1]').click()

    await expect(page.getByText('Automatically download new episodes')).toBeVisible()
    await expect(page.getByText('Automatically update podcasts')).toBeVisible()
    await expect(page.getByText('Auto-transcribe', { exact: true })).toBeVisible()
})
