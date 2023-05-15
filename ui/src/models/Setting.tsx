export type Setting = {
    id: number,
    autoDownload: boolean,
    autoUpdate: boolean,
    autoCleanup: boolean,
    autoCleanupDays: number,
    podcastPrefill: number,
    useExistingFilenames: boolean,
    replaceInvalidCharacters: boolean,
    replacementStrategy: string,
    episodeFormat: string,
    podcastFormat: string
}
