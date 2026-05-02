import {components} from "../../schema";

export const generatePodcastDefaultSettings = (podcastId: number) => {
    return {
        activated: false,
        autoCleanup: false,
        autoCleanupDays: 0,
        autoDownload: true,
        autoUpdate: true,
        directPaths: false,
        episodeFormat: "",
        episodeNumbering: false,
        podcastFormat: "",
        podcastId: podcastId,
        podcastPrefill: 0,
        replaceInvalidCharacters: false,
        replacementStrategy: "replace-with-dash-and-underscore",
        useExistingFilename: false,
        useOneCoverForAllEpisodes: false,
        sponsorblockEnabled: false,
        sponsorblockCategories: []
    } satisfies components['schemas']['PodcastSetting']
}