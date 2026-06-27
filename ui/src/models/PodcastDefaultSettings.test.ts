import { describe, expect, it } from 'vitest'
import { generatePodcastDefaultSettings } from './PodcastDefaultSettings'
import { Setting } from './Setting'

const globalSettings: Setting = {
    id: 'global',
    episodeNumbering: true,
    autoDownload: false,
    autoUpdate: false,
    autoCleanup: true,
    autoCleanupDays: 14,
    podcastPrefill: 5,
    useExistingFilename: true,
    replaceInvalidCharacters: true,
    replacementStrategy: 'remove',
    episodeFormat: '{episodeTitle}',
    podcastFormat: '{podcastTitle}',
    directPaths: true,
    autoTranscodeOpus: false,
    useOneCoverForAllEpisodes: true,
    nfoFormat: 'album',
    coverFilename: 'cover',
    maxParallelDownloads: 3,
}

describe('generatePodcastDefaultSettings', () => {
    it('inherits the inheritable fields from the global settings', () => {
        const result = generatePodcastDefaultSettings('podcast-1', globalSettings)

        expect(result.podcastId).toBe('podcast-1')
        // never auto-activated: the user must explicitly opt into an override
        expect(result.activated).toBe(false)
        // fields the bug report is about must reflect the global defaults
        expect(result.nfoFormat).toBe('album')
        expect(result.coverFilename).toBe('cover')
        expect(result.useOneCoverForAllEpisodes).toBe(true)
        // the rest of the overlapping fields inherit too
        expect(result.episodeNumbering).toBe(true)
        expect(result.autoDownload).toBe(false)
        expect(result.autoUpdate).toBe(false)
        expect(result.autoCleanup).toBe(true)
        expect(result.autoCleanupDays).toBe(14)
        expect(result.podcastPrefill).toBe(5)
        expect(result.useExistingFilename).toBe(true)
        expect(result.replaceInvalidCharacters).toBe(true)
        expect(result.replacementStrategy).toBe('remove')
        expect(result.episodeFormat).toBe('{episodeTitle}')
        expect(result.podcastFormat).toBe('{podcastTitle}')
        expect(result.directPaths).toBe(true)
    })

    it('falls back to the built-in defaults when global settings are unavailable', () => {
        const result = generatePodcastDefaultSettings('podcast-2')

        expect(result.podcastId).toBe('podcast-2')
        expect(result.activated).toBe(false)
        expect(result.nfoFormat).toBe('off')
        expect(result.coverFilename).toBe('image')
        expect(result.useOneCoverForAllEpisodes).toBe(false)
        expect(result.autoDownload).toBe(true)
        expect(result.autoUpdate).toBe(true)
        expect(result.replacementStrategy).toBe('replace-with-dash-and-underscore')
    })
})
