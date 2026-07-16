import { request } from '@playwright/test'
import fs from 'node:fs'
import path from 'node:path'
import { FEED_BASE, SEED_FILE, type Seed } from './helpers'

const BASE_URL = 'http://127.0.0.1:8000'

const sleep = (ms: number) => new Promise(resolve => setTimeout(resolve, ms))

/**
 * Seeds the freshly started server once for the whole suite: adds the fixture
 * podcast by feed URL, then waits until both initial episodes are
 * auto-downloaded and episode 1's feed transcript is parsed. Tests read the
 * resulting ids from SEED_FILE and only drive UI flows.
 */
export default async function globalSetup() {
    const api = await request.newContext({ baseURL: BASE_URL })

    const addResponse = await api.post('/api/v1/podcasts/feed', {
        data: { rssFeedUrl: `${FEED_BASE}/feed.xml` },
    })
    if (!addResponse.ok()) {
        throw new Error(`could not add fixture podcast: ${addResponse.status()} ${await addResponse.text()}`)
    }
    const podcast = await addResponse.json()

    const episodeIdsByTitle: Record<string, string> = {}
    for (let attempt = 0; attempt < 90 && Object.keys(episodeIdsByTitle).length < 2; attempt++) {
        const episodes = await (
            await api.get(`/api/v1/podcasts/${podcast.id}/episodes?only_unlistened=false`)
        ).json()
        for (const e of episodes) {
            if (e.podcastEpisode.status) {
                episodeIdsByTitle[e.podcastEpisode.name] = e.podcastEpisode.id
            }
        }
        if (Object.keys(episodeIdsByTitle).length < 2) {
            await sleep(1000)
        }
    }
    if (Object.keys(episodeIdsByTitle).length < 2) {
        throw new Error(`fixture episodes were not downloaded within 90s: ${JSON.stringify(episodeIdsByTitle)}`)
    }

    const smokeEpisodeId = episodeIdsByTitle['Smoke Episode 1']
    let transcriptParsed = false
    for (let attempt = 0; attempt < 60 && !transcriptParsed; attempt++) {
        const transcripts = await (
            await api.get(`/api/v1/podcasts/episodes/${smokeEpisodeId}/transcripts`)
        ).json()
        transcriptParsed = transcripts.some((t: any) => t.source === 'feed' && t.status === 'parsed')
        if (!transcriptParsed) {
            await sleep(1000)
        }
    }
    if (!transcriptParsed) {
        throw new Error('feed transcript was not parsed within 60s')
    }

    const seed: Seed = { podcastId: podcast.id, episodeIdsByTitle }
    fs.mkdirSync(path.dirname(SEED_FILE), { recursive: true })
    fs.writeFileSync(SEED_FILE, JSON.stringify(seed))
    await api.dispose()
}
