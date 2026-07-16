import {components} from "../../schema";

/**
 * Build the per-podcast settings shown when a podcast has no explicit override
 * row yet. The override is never auto-activated (`activated: false`), but the
 * displayed values inherit from the instance-wide settings so the modal matches
 * what the backend actually applies as the fallback. When the global settings
 * are unavailable the built-in defaults are used instead.
 */
export const generatePodcastDefaultSettings = (
    podcastId: string,
    globalSettings?: components['schemas']['Setting']
) => {
    return {
        activated: false,
        podcastId: podcastId,
        autoCleanup: globalSettings?.autoCleanup ?? false,
        autoCleanupDays: globalSettings?.autoCleanupDays ?? 0,
        autoDownload: globalSettings?.autoDownload ?? true,
        autoUpdate: globalSettings?.autoUpdate ?? true,
        directPaths: globalSettings?.directPaths ?? false,
        episodeFormat: globalSettings?.episodeFormat ?? "",
        episodeNumbering: globalSettings?.episodeNumbering ?? false,
        podcastFormat: globalSettings?.podcastFormat ?? "",
        podcastPrefill: globalSettings?.podcastPrefill ?? 0,
        replaceInvalidCharacters: globalSettings?.replaceInvalidCharacters ?? false,
        replacementStrategy: globalSettings?.replacementStrategy ?? "replace-with-dash-and-underscore",
        useExistingFilename: globalSettings?.useExistingFilename ?? false,
        useOneCoverForAllEpisodes: globalSettings?.useOneCoverForAllEpisodes ?? false,
        nfoFormat: globalSettings?.nfoFormat ?? "off",
        coverFilename: globalSettings?.coverFilename ?? "image",
        autoTranscribe: false
    } satisfies components['schemas']['PodcastSetting']
}
