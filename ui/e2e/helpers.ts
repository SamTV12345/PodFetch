import type { Locator, Page } from '@playwright/test'
import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

export const SEED_FILE = path.resolve(
    path.dirname(fileURLToPath(import.meta.url)),
    '../../.e2e-run/seed.json'
)

export const FEED_BASE = 'http://127.0.0.1:9123'
export const WHISPER_BASE = 'http://127.0.0.1:9998'

export type Seed = { podcastId: string; episodeIdsByTitle: Record<string, string> }

export const readSeed = (): Seed => JSON.parse(fs.readFileSync(SEED_FILE, 'utf-8'))

/** The row of an episode on the podcast detail page, located by its title. */
export const episodeRow = (page: Page, title: string): Locator =>
    page.locator('div[id^="episode_"]', { hasText: title })

/** Starts playback of an episode via the big play button in its row. */
export const playEpisode = async (page: Page, title: string) => {
    await episodeRow(page, title).locator('svg.lucide-circle-play').click()
}

/** Opens the detailed player through the maximize overlay of the bottom bar. */
export const openDetailedPlayer = async (page: Page) => {
    const maximize = page.locator('svg.lucide-maximize-2')
    await maximize.waitFor({ state: 'visible' })
    await maximize.locator('..').click({ force: true })
}

export const audioCurrentTime = (page: Page) =>
    page.evaluate(() => {
        const player = document.getElementById('audio-player') as HTMLMediaElement | null
        return player?.currentTime ?? -1
    })

/** Switches the episode search page into transcript mode and searches. */
export const searchTranscripts = async (page: Page, query: string) => {
    await page.goto('/ui/podcasts/search')
    await page.getByRole('button', { name: 'Transcripts' }).click()
    await page.locator('#transcript-search-input').fill(query)
}
