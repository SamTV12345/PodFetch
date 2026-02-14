import * as FileSystem from 'expo-file-system/legacy';
import { offlineDB, DownloadedEpisode } from '@/store/offlineStore';
import { components } from '@/schema';
import { useStore } from '@/store/store';
import { getAuthenticatedMediaUrl } from '@/utils/mediaUrl';

/**
 * DownloadManager - Verwaltet Episode-Downloads für Offline-Nutzung
 */

export interface DownloadProgress {
    episodeId: string;
    progress: number;           // 0-1
    totalBytes: number;
    downloadedBytes: number;
    status: 'pending' | 'downloading' | 'completed' | 'failed' | 'cancelled';
    error?: string;
}

type DownloadProgressCallback = (progress: DownloadProgress) => void;

class DownloadManagerClass {
    private activeDownloads: Map<string, FileSystem.DownloadResumable> = new Map();
    private progressCallbacks: Map<string, Set<DownloadProgressCallback>> = new Map();
    private downloadStatuses: Map<string, DownloadProgress> = new Map();

    private getEpisodeDirectory(): string {
        return `${FileSystem.documentDirectory}episodes/`;
    }

    private getEpisodeFilePath(episodeId: string, extension: string = 'mp3'): string {
        const safeId = episodeId.replace(/[^a-zA-Z0-9-_]/g, '_');
        return `${this.getEpisodeDirectory()}${safeId}.${extension}`;
    }

    async ensureDirectoryExists(): Promise<void> {
        const dirInfo = await FileSystem.getInfoAsync(this.getEpisodeDirectory());
        if (!dirInfo.exists) {
            await FileSystem.makeDirectoryAsync(this.getEpisodeDirectory(), { intermediates: true });
        }
    }

    async downloadEpisode(
        episode: components['schemas']['PodcastEpisodeDto'],
        podcast: components['schemas']['PodcastDto']
    ): Promise<void> {
        const episodeId = episode.episode_id;

        const existing = await offlineDB.getDownloadedEpisode(episodeId);
        if (existing) {
            const fileInfo = await FileSystem.getInfoAsync(existing.localPath);
            if (fileInfo.exists) {
                console.log(`Episode ${episodeId} already downloaded`);
                return;
            }
            await offlineDB.deleteDownloadedEpisode(episodeId);
        }

        if (this.activeDownloads.has(episodeId)) {
            console.log(`Download for ${episodeId} already in progress`);
            return;
        }

        await this.ensureDirectoryExists();

        const urlParts = episode.url.split('.');
        const extension = urlParts[urlParts.length - 1]?.split('?')[0] || 'mp3';
        const localPath = this.getEpisodeFilePath(episodeId, extension);

        this.updateProgress(episodeId, {
            episodeId,
            progress: 0,
            totalBytes: 0,
            downloadedBytes: 0,
            status: 'pending'
        });

        const serverUrl = useStore.getState().serverUrl;
        const authType = useStore.getState().authType;
        const userApiKey = useStore.getState().userApiKey;
        let downloadUrl = episode.url;

        const isAbsoluteUrl = (url: string) => url.startsWith('http://') || url.startsWith('https://');

        const apiKey = authType === 'basic' ? userApiKey : null;

        if (episode.local_url && serverUrl && !isAbsoluteUrl(episode.local_url)) {
            downloadUrl = getAuthenticatedMediaUrl(episode.local_url, serverUrl, apiKey);
        } else if (episode.local_url && isAbsoluteUrl(episode.local_url)) {
            downloadUrl = episode.local_url;
        } else if (episode.url.startsWith('/') && serverUrl) {
            downloadUrl = getAuthenticatedMediaUrl(episode.url, serverUrl, apiKey);
        } else if (isAbsoluteUrl(episode.url)) {
            downloadUrl = episode.url;
        }

        try {
            const downloadResumable = FileSystem.createDownloadResumable(
                downloadUrl,
                localPath,
                {},
                (downloadProgress) => {
                    const progress = downloadProgress.totalBytesExpectedToWrite > 0
                        ? downloadProgress.totalBytesWritten / downloadProgress.totalBytesExpectedToWrite
                        : 0;

                    this.updateProgress(episodeId, {
                        episodeId,
                        progress,
                        totalBytes: downloadProgress.totalBytesExpectedToWrite,
                        downloadedBytes: downloadProgress.totalBytesWritten,
                        status: 'downloading'
                    });
                }
            );

            this.activeDownloads.set(episodeId, downloadResumable);
            this.updateProgress(episodeId, {
                episodeId,
                progress: 0,
                totalBytes: 0,
                downloadedBytes: 0,
                status: 'downloading'
            });

            const result = await downloadResumable.downloadAsync();

            if (result?.uri) {
                const fileInfo = await FileSystem.getInfoAsync(result.uri);

                await offlineDB.saveDownloadedEpisode({
                    episodeId: episode.episode_id,
                    podcastId: episode.podcast_id,
                    name: episode.name,
                    localPath: result.uri,
                    originalUrl: episode.url,
                    imageUrl: episode.image_url,
                    totalTime: episode.total_time,
                    downloadedAt: new Date().toISOString(),
                    fileSize: (fileInfo as any).size || 0,
                    podcastName: podcast.name,
                    podcastImageUrl: podcast.image_url
                });

                this.updateProgress(episodeId, {
                    episodeId,
                    progress: 1,
                    totalBytes: (fileInfo as any).size || 0,
                    downloadedBytes: (fileInfo as any).size || 0,
                    status: 'completed'
                });

                console.log(`Episode ${episodeId} downloaded successfully`);
            }
        } catch (error) {
            console.error(`Download failed for ${episodeId}:`, error);
            this.updateProgress(episodeId, {
                episodeId,
                progress: 0,
                totalBytes: 0,
                downloadedBytes: 0,
                status: 'failed',
                error: error instanceof Error ? error.message : 'Download failed'
            });
            throw error;
        } finally {
            this.activeDownloads.delete(episodeId);
        }
    }

    async cancelDownload(episodeId: string): Promise<void> {
        const download = this.activeDownloads.get(episodeId);
        if (download) {
            await download.pauseAsync();
            this.activeDownloads.delete(episodeId);

            const partialPath = this.getEpisodeFilePath(episodeId);
            try {
                const fileInfo = await FileSystem.getInfoAsync(partialPath);
                if (fileInfo.exists) {
                    await FileSystem.deleteAsync(partialPath);
                }
            } catch (e) {
                console.log(e)
            }

            this.updateProgress(episodeId, {
                episodeId,
                progress: 0,
                totalBytes: 0,
                downloadedBytes: 0,
                status: 'cancelled'
            });
        }
    }


    async deleteDownload(episodeId: string): Promise<void> {
        await this.cancelDownload(episodeId);

        const episode = await offlineDB.getDownloadedEpisode(episodeId);
        if (episode) {
            try {
                const fileInfo = await FileSystem.getInfoAsync(episode.localPath);
                if (fileInfo.exists) {
                    await FileSystem.deleteAsync(episode.localPath);
                }
            } catch (e) {
                console.warn(`Could not delete file for ${episodeId}:`, e);
            }

            await offlineDB.deleteDownloadedEpisode(episodeId);
        }

        this.downloadStatuses.delete(episodeId);
    }

    async isDownloaded(episodeId: string): Promise<boolean> {
        const episode = await offlineDB.getDownloadedEpisode(episodeId);
        if (!episode) return false;

        // Prüfe ob Datei noch existiert
        const fileInfo = await FileSystem.getInfoAsync(episode.localPath);
        if (!fileInfo.exists) {
            // Datei wurde gelöscht, bereinige DB
            await offlineDB.deleteDownloadedEpisode(episodeId);
            return false;
        }

        return true;
    }


    async getLocalPath(episodeId: string): Promise<string | null> {
        const episode = await offlineDB.getDownloadedEpisode(episodeId);
        if (!episode) return null;

        const fileInfo = await FileSystem.getInfoAsync(episode.localPath);
        if (!fileInfo.exists) {
            await offlineDB.deleteDownloadedEpisode(episodeId);
            return null;
        }

        return episode.localPath;
    }

    /**
     * Holt alle heruntergeladenen Episoden
     */
    async getAllDownloads(): Promise<DownloadedEpisode[]> {
        return offlineDB.getAllDownloadedEpisodes();
    }

    /**
     * Holt Downloads für einen bestimmten Podcast
     */
    async getDownloadsForPodcast(podcastId: number): Promise<DownloadedEpisode[]> {
        return offlineDB.getDownloadedEpisodesByPodcast(podcastId);
    }

    /**
     * Berechnet den Gesamtspeicherplatz der Downloads
     */
    async getTotalDownloadSize(): Promise<number> {
        return offlineDB.getTotalDownloadSize();
    }

    /**
     * Holt die Anzahl der Downloads
     */
    async getDownloadCount(): Promise<number> {
        return offlineDB.getDownloadCount();
    }

    /**
     * Registriert einen Callback für Download-Progress-Updates
     */
    subscribeToProgress(episodeId: string, callback: DownloadProgressCallback): () => void {
        if (!this.progressCallbacks.has(episodeId)) {
            this.progressCallbacks.set(episodeId, new Set());
        }
        this.progressCallbacks.get(episodeId)!.add(callback);

        // Sende aktuellen Status sofort
        const currentStatus = this.downloadStatuses.get(episodeId);
        if (currentStatus) {
            callback(currentStatus);
        }

        // Return unsubscribe function
        return () => {
            this.progressCallbacks.get(episodeId)?.delete(callback);
        };
    }

    /**
     * Holt den aktuellen Download-Status
     */
    getDownloadStatus(episodeId: string): DownloadProgress | undefined {
        return this.downloadStatuses.get(episodeId);
    }

    /**
     * Prüft ob ein Download gerade läuft
     */
    isDownloading(episodeId: string): boolean {
        return this.activeDownloads.has(episodeId);
    }

    private updateProgress(episodeId: string, progress: DownloadProgress): void {
        this.downloadStatuses.set(episodeId, progress);

        const callbacks = this.progressCallbacks.get(episodeId);
        if (callbacks) {
            callbacks.forEach(cb => cb(progress));
        }
    }

    async clearAllDownloads(): Promise<void> {
        for (const [episodeId] of this.activeDownloads) {
            await this.cancelDownload(episodeId);
        }

        const downloads = await this.getAllDownloads();
        for (const download of downloads) {
            await this.deleteDownload(download.episodeId)
        }
    }
}

// Singleton-Instanz
export const downloadManager = new DownloadManagerClass();
