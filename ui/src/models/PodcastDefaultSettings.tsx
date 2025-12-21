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
        useExistingFilename: false
    } satisfies components['schemas']['PodcastSetting']
}