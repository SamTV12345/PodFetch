import { expect, test } from '@playwright/test'
import { readSeed } from '../helpers'

const seed = readSeed()

test('favoriting a podcast lists it on the favorites page', async ({ page }) => {
    await page.goto('/ui/podcasts')
    await expect(page.getByText('Transcript E2E Podcast', { exact: true })).toBeVisible()

    // The sidebar's Favorites entry also uses a heart icon; the card's heart
    // is the absolutely positioned one on the cover image.
    await page.locator('svg.lucide-heart.absolute').click()

    await expect(async () => {
        await page.goto('/ui/favorites')
        await expect(page.getByText('Transcript E2E Podcast', { exact: true })).toBeVisible({
            timeout: 2_000,
        })
    }).toPass({ timeout: 15_000 })

    // Unfavor again so the favorites page returns to its empty state.
    await page.locator('svg.lucide-heart.absolute').click()
    await expect(async () => {
        await page.goto('/ui/favorites')
        await expect(page.getByText('Transcript E2E Podcast', { exact: true })).not.toBeVisible({
            timeout: 2_000,
        })
    }).toPass({ timeout: 15_000 })
})

test('timeline shows the downloaded fixture episodes', async ({ page }) => {
    await page.goto('/ui/timeline')

    // "Only favorite podcasts" is on by default; the fixture podcast is not
    // favored, so widen the filter first.
    await page
        .getByText('Only favorite podcasts', { exact: true })
        .locator('xpath=following-sibling::*[1]')
        .click()

    await expect(page.getByText('Smoke Episode 1', { exact: true }).first()).toBeVisible()
})

test('episodes archive lists downloaded episodes', async ({ page }) => {
    await page.goto('/ui/episodes')

    await expect(page.getByText('Smoke Episode 1', { exact: true }).first()).toBeVisible()
    await expect(page.getByText('Plain Episode 2', { exact: true }).first()).toBeVisible()
})

test('statistics page renders its charts', async ({ page }) => {
    await page.goto('/ui/stats')

    // The listening-behavior chart axes carry translated weekday labels.
    await expect(page.getByText('Monday').first()).toBeVisible()
})

test('inbox page renders', async ({ page }) => {
    await page.goto('/ui/inbox')

    await expect(page.getByRole('heading').first()).toBeVisible()
})

test('a playlist can be created through the three-step wizard', async ({ page }) => {
    await page.goto('/ui/home/playlist')

    await page.getByRole('button', { name: 'Add new' }).click()
    const dialog = page.getByRole('dialog')

    // Stage 1: name.
    await dialog.locator('#playlist-name').fill('E2E Test Playlist')
    await dialog.getByRole('button', { name: 'Next' }).click()

    // Stage 2: pick an episode via the embedded search.
    await dialog.locator('#search-input').fill('Smoke Episode')
    await dialog.getByText('Smoke Episode 1', { exact: true }).first().click()
    await dialog.getByRole('button', { name: 'Next' }).click()

    // Stage 3: the review step names the playlist; submit it.
    await expect(dialog.getByText('E2E Test Playlist').first()).toBeVisible()
    await dialog.getByRole('button', { name: 'Create playlist' }).click()

    await expect(page.getByText('E2E Test Playlist', { exact: true }).first()).toBeVisible({
        timeout: 15_000,
    })
    await expect(page.getByText('1 items').first()).toBeVisible()

    // Opening the playlist shows its episode ("Open" exact, to not match the
    // header's "Open notifications" button).
    await page.getByRole('button', { name: 'Open', exact: true }).first().click()
    await expect(page.getByText('Smoke Episode 1', { exact: true }).first()).toBeVisible()
})

test('user menu exposes profile and administration entries', async ({ page }) => {
    await page.goto(`/ui/podcasts/${seed.podcastId}/episodes`)

    await page.locator('svg.lucide-circle-user-round').first().locator('xpath=ancestor::button[1]').click()
    await expect(page.getByText('Profile', { exact: true })).toBeVisible()
    await expect(page.getByText('System information', { exact: true })).toBeVisible()
    await expect(page.getByText('User Administration', { exact: true })).toBeVisible()
})
