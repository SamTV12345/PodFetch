export type Setting = {
    id: number,
    autoDownload: boolean,
    autoUpdate: boolean,
    autoCleanup: boolean,
    autoCleanupDays: number,
    podcastPrefill: number,
    useExistingFilename: boolean,
    replaceInvalidCharacters: boolean,
    replacementStrategy: string,
    episodeFormat: string,
    podcastFormat: string
}
