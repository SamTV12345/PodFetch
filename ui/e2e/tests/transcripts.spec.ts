import { expect, test } from '@playwright/test'
import {
    FEED_BASE,
    WHISPER_BASE,
    audioCurrentTime,
    episodeRow,
    openDetailedPlayer,
    playEpisode,
    readSeed,
    searchTranscripts,
} from '../helpers'

const seed = readSeed()
const detailPage = `/ui/podcasts/${seed.podcastId}/episodes`

// Tests share one server and run in file order (workers = 1). Order matters
// for the whisper-flow tests at the end: the failure/retry test must run
// BEFORE auto-transcribe is enabled, and the "no transcript" check must run
// before Plain Episode 2 gets its generated transcript.

test('player shows the transcript tab with segments and click-to-seek', async ({ page }) => {
    await page.goto(detailPage)
    await playEpisode(page, 'Smoke Episode 1')
    await openDetailedPlayer(page)

    await page.getByText('Transcript', { exact: true }).click()

    // Feed transcript segments incl. speaker from the VTT voice tag.
    await expect(page.getByText('Welcome to the transcript smoke test episode.')).toBeVisible()
    await expect(page.getByText('Alice:', { exact: false })).toBeVisible()

    // Clicking a timestamped segment seeks the audio element.
    await page.getByText('We are talking about the zephyrquark keyword for full-text search.').click()
    await expect.poll(() => audioCurrentTime(page)).toBeGreaterThanOrEqual(4)
})

test('transcript tab highlights the active segment and offers auto-scroll', async ({ page }) => {
    await page.goto(detailPage)
    await playEpisode(page, 'Smoke Episode 1')
    await openDetailedPlayer(page)
    await page.getByText('Transcript', { exact: true }).click()

    const autoScroll = page.getByLabel('Auto-scroll')
    await expect(autoScroll).toBeChecked()
    await autoScroll.uncheck()
    await expect(autoScroll).not.toBeChecked()

    // Seek into segment 2 and expect exactly that segment to be highlighted.
    const segment2 = page
        .locator('li')
        .filter({ hasText: 'We are talking about the zephyrquark keyword' })
    await segment2.click()
    await expect(segment2).toHaveClass(/ui-text-accent/)
    await expect(
        page.locator('li').filter({ hasText: 'Welcome to the transcript smoke test episode.' })
    ).not.toHaveClass(/ui-text-accent/)
})

test('player shows a hint when the episode has no transcript', async ({ page }) => {
    await page.goto(detailPage)
    await playEpisode(page, 'Plain Episode 2')
    await openDetailedPlayer(page)
    await page.getByText('Transcript', { exact: true }).click()

    await expect(page.getByText('No transcript available')).toBeVisible()
})

test('episode search finds spoken words in transcript mode', async ({ page }) => {
    await searchTranscripts(page, 'zephyrquark')

    // Episode card with a highlighted snippet and a timestamp badge.
    await expect(page.getByText('Smoke Episode 1', { exact: true })).toBeVisible()
    await expect(page.locator('b', { hasText: 'zephyrquark' })).toBeVisible()
    await expect(page.getByText('0:04')).toBeVisible()
})

test('transcript search handles empty input and misses gracefully', async ({ page }) => {
    await searchTranscripts(page, 'kzzqwhatever-no-such-term')
    await expect(page.getByText('No results found for')).toBeVisible()

    await page.locator('#transcript-search-input').fill('')
    await expect(page.getByText('No results found for')).not.toBeVisible()

    // Switching back to metadata mode keeps the classic search working.
    await page.getByRole('button', { name: 'Title/Description' }).click()
    await page.locator('#search-input').fill('Smoke Episode')
    await expect(page.getByText('Smoke Episode 1', { exact: true })).toBeVisible()
})

test('clicking a search hit starts playback at the matched position', async ({ page }) => {
    await searchTranscripts(page, 'zephyrquark')

    await page.locator('b', { hasText: 'zephyrquark' }).click()

    // The bottom player bar appears with the episode and the audio is seeked
    // to the hit's start (4s).
    await expect.poll(() => audioCurrentTime(page), { timeout: 15_000 }).toBeGreaterThanOrEqual(4)
})

test('transcript snippets escape embedded markup instead of executing it', async ({ page }) => {
    await searchTranscripts(page, 'xanadu99')

    await expect(page.locator('b', { hasText: 'xanadu99' })).toBeVisible()
    // The transcript text contains a <script> tag; it must never execute.
    expect(await page.evaluate(() => (window as any).__xss)).toBeUndefined()
})

test('generated rss feed re-exports the transcript tag', async ({ page }) => {
    const rss = await (await page.request.get(`/rss/${seed.podcastId}`)).text()

    expect(rss).toContain('xmlns:podcast="https://podcastindex.org/namespace/1.0"')
    expect(rss).toContain('<podcast:transcript')
    expect(rss).toContain('type="text/vtt"')
    expect(rss).toContain(
        `/api/v1/podcasts/episodes/${seed.episodeIdsByTitle['Smoke Episode 1']}/transcripts/`
    )
})

test('transcribe action enqueues a whisper job and reports its status', async ({ page }) => {
    await page.goto(detailPage)

    const transcribeIcon = episodeRow(page, 'Plain Episode 2').getByTestId('transcribe-episode')
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
    await searchTranscripts(page, 'quixotron')
    await expect(page.locator('b', { hasText: 'quixotron' })).toBeVisible()
})

test('failed whisper jobs surface as failed and can be retried', async ({ page }) => {
    // Publish a fresh episode and let podfetch pick it up.
    await page.request.post(`${FEED_BASE}/control/publish`, { data: { title: 'Late Episode 3' } })
    await page.request.post(`/api/v1/podcasts/${seed.podcastId}/refresh`)

    await page.goto(detailPage)
    const row = episodeRow(page, 'Late Episode 3')
    const transcribeIcon = row.getByTestId('transcribe-episode')
    // The icon appears once the episode is downloaded.
    await expect(transcribeIcon).toBeVisible({ timeout: 60_000 })

    await page.request.post(`${WHISPER_BASE}/control/mode`, { data: { mode: 'fail' } })
    await transcribeIcon.click()

    // Three fast attempts against the failing mock, then the job is failed.
    await expect(transcribeIcon).toHaveAttribute('aria-label', 'Transcription failed', {
        timeout: 60_000,
    })

    // Retry once the backend is healthy again.
    await page.request.post(`${WHISPER_BASE}/control/mode`, { data: { mode: 'ok' } })
    await transcribeIcon.click()
    await expect(transcribeIcon).toHaveAttribute('aria-label', 'Transcript', {
        timeout: 60_000,
    })
})

test('auto-transcribe transcribes new downloads without user interaction', async ({ page }) => {
    await page.goto(detailPage)

    // Enable auto-transcribe in the podcast settings modal...
    await page.locator('svg.lucide-settings').locator('xpath=ancestor::button[1]').click()
    const toggleRow = page.locator('label', { hasText: 'Auto-transcribe' })
    await expect(toggleRow).toBeVisible()
    await toggleRow.locator('xpath=following-sibling::*[1]').click()
    const saveButton = page.getByRole('button', { name: 'Save' })
    await saveButton.scrollIntoViewIfNeeded()
    await saveButton.click()

    // ...and verify it persisted server-side: a fresh page load re-fetches
    // the settings from the API.
    await page.reload()
    await page.locator('svg.lucide-settings').locator('xpath=ancestor::button[1]').click()
    await expect(
        page.locator('label', { hasText: 'Auto-transcribe' }).locator('xpath=following-sibling::*[1]//input')
    ).toBeChecked()
    await page.keyboard.press('Escape')

    // Publish a new episode; download + transcription must happen on their own.
    await page.request.post(`${FEED_BASE}/control/publish`, { data: { title: 'Auto Episode 4' } })
    await page.request.post(`/api/v1/podcasts/${seed.podcastId}/refresh`)

    // The episode list doesn't live-insert brand-new episodes, so reload
    // until the downloaded episode (and with it the transcribe badge) shows up.
    const transcribeIcon = episodeRow(page, 'Auto Episode 4').getByTestId('transcribe-episode')
    await expect(async () => {
        await page.reload()
        await expect(transcribeIcon).toBeVisible({ timeout: 3_000 })
    }).toPass({ timeout: 90_000 })

    await expect(transcribeIcon).toHaveAttribute('aria-label', 'Transcript', { timeout: 90_000 })
})
