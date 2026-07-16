import { request } from '@playwright/test'
import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const BASE_URL = 'http://127.0.0.1:8000'
const FEED_URL = 'http://127.0.0.1:9123/feed.xml'
export const SEED_FILE = path.resolve(
    path.dirname(fileURLToPath(import.meta.url)),
    '../../.e2e-run/seed.json'
)

const sleep = (ms: number) => new Promise(resolve => setTimeout(resolve, ms))

/**
 * Seeds the freshly started server once for the whole suite: adds the fixture
 * podcast by feed URL, then waits until the episode is auto-downloaded and
 * its feed transcript is parsed. Tests read the resulting ids from SEED_FILE
 * and only drive UI flows.
 */
export default async function globalSetup() {
    const api = await request.newContext({ baseURL: BASE_URL })

    const addResponse = await api.post('/api/v1/podcasts/feed', {
        data: { rssFeedUrl: FEED_URL },
    })
    if (!addResponse.ok()) {
        throw new Error(`could not add fixture podcast: ${addResponse.status()} ${await addResponse.text()}`)
    }
    const podcast = await addResponse.json()

    let episodeId: string | undefined
    for (let attempt = 0; attempt < 90 && !episodeId; attempt++) {
        const episodes = await (
            await api.get(`/api/v1/podcasts/${podcast.id}/episodes?only_unlistened=false`)
        ).json()
        episodeId = episodes.find((e: any) => e.podcastEpisode.status)?.podcastEpisode.id
        if (!episodeId) {
            await sleep(1000)
        }
    }
    if (!episodeId) {
        throw new Error('fixture episode was not downloaded within 90s')
    }

    let transcriptParsed = false
    for (let attempt = 0; attempt < 60 && !transcriptParsed; attempt++) {
        const transcripts = await (
            await api.get(`/api/v1/podcasts/episodes/${episodeId}/transcripts`)
        ).json()
        transcriptParsed = transcripts.some((t: any) => t.source === 'feed' && t.status === 'parsed')
        if (!transcriptParsed) {
            await sleep(1000)
        }
    }
    if (!transcriptParsed) {
        throw new Error('feed transcript was not parsed within 60s')
    }

    fs.mkdirSync(path.dirname(SEED_FILE), { recursive: true })
    fs.writeFileSync(SEED_FILE, JSON.stringify({ podcastId: podcast.id, episodeId }))
    await api.dispose()
}
