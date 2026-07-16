import { expect, test } from '@playwright/test'
import fs from 'node:fs'
import { SEED_FILE } from '../global-setup'

const seed = JSON.parse(fs.readFileSync(SEED_FILE, 'utf-8')) as {
    podcastId: string
    episodeId: string
}

const detailPage = `/ui/podcasts/${seed.podcastId}/episodes`

test.describe('transcript features', () => {
    test('player shows the transcript tab with segments and click-to-seek', async ({ page }) => {
        await page.goto(detailPage)

        // Start playback via the episode row's big play button.
        await page.locator('svg.lucide-circle-play').first().click()

        // Open the detailed player through the maximize overlay on the
        // now-visible bottom bar thumbnail (hover overlay, so force).
        const maximize = page.locator('svg.lucide-maximize-2')
        await expect(maximize).toBeVisible()
        await maximize.locator('..').click({ force: true })

        await page.getByText('Transcript', { exact: true }).click()

        // Feed transcript segments incl. speaker from the VTT voice tag.
        await expect(page.getByText('Welcome to the transcript smoke test episode.')).toBeVisible()
        await expect(page.getByText('Alice:', { exact: false })).toBeVisible()

        // Clicking a timestamped segment seeks the audio element.
        await page.getByText('We are talking about the zephyrquark keyword for full-text search.').click()
        await expect
            .poll(async () =>
                page.evaluate(() => {
                    const player = document.getElementById('audio-player') as HTMLMediaElement | null
                    return player?.currentTime ?? -1
                })
            )
            .toBeGreaterThanOrEqual(4)
    })

    test('episode search finds spoken words in transcript mode', async ({ page }) => {
        await page.goto('/ui/podcasts/search')

        await page.getByRole('button', { name: 'Transcripts' }).click()
        await page.locator('#transcript-search-input').fill('zephyrquark')

        // Episode card with a highlighted snippet.
        await expect(page.getByText('Smoke Episode 1')).toBeVisible()
        await expect(page.locator('b', { hasText: 'zephyrquark' })).toBeVisible()
    })

    test('transcribe action enqueues a whisper job and reports its status', async ({ page }) => {
        await page.goto(detailPage)

        const transcribeIcon = page.getByTestId('transcribe-episode')
        await expect(transcribeIcon).toBeVisible()
        await expect(transcribeIcon).toHaveAttribute('aria-label', 'Transcribe')

        await transcribeIcon.click()

        // Badge flips to pending as soon as the job row exists...
        await expect(transcribeIcon).toHaveAttribute('aria-label', 'Transcription pending', {
            timeout: 15_000,
        })
        // ...and to the done state once the worker stored the generated
        // transcript (worker polls every 15s, mock whisper answers after ~1s).
        await expect(transcribeIcon).toHaveAttribute('aria-label', 'Transcript', {
            timeout: 45_000,
        })

        // The generated content is immediately searchable.
        await page.goto('/ui/podcasts/search')
        await page.getByRole('button', { name: 'Transcripts' }).click()
        await page.locator('#transcript-search-input').fill('quixotron')
        await expect(page.locator('b', { hasText: 'quixotron' })).toBeVisible()
    })

    test('podcast settings expose the auto-transcribe toggle', async ({ page }) => {
        await page.goto(detailPage)

        await page.locator('svg.lucide-settings').locator('xpath=ancestor::button[1]').click()

        await expect(page.getByText('Auto-transcribe')).toBeVisible()
    })
})
