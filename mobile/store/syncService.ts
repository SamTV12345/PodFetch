import * as Network from 'expo-network';
import { offlineDB, LocalWatchProgress } from '@/store/offlineStore';
import { useStore } from '@/store/store';
import { components, paths } from '@/schema';
import createFetchClient from 'openapi-fetch';

/**
 * SyncService - Synchronisiert lokale Daten mit dem PodFetch-Server
 *
 * Features:
 * - Automatische Sync wenn online
 * - Queue-basierte Sync für Offline-Änderungen
 * - Conflict Resolution (lokale Änderung gewinnt wenn neuer)
 */

export interface SyncResult {
    success: boolean;
    syncedItems: number;
    failedItems: number;
    errors: string[];
}

class SyncServiceClass {
    private isSyncing: boolean = false;
    private syncInterval: ReturnType<typeof setInterval> | null = null;
    private listeners: Set<(isOnline: boolean) => void> = new Set();
    private lastOnlineState: boolean | null = null;


    private getAuthenticatedClient() {
        const state = useStore.getState();
        const serverUrl = state.serverUrl;

        if (!serverUrl) return null;

        const headers: Record<string, string> = {};

        if (state.authType === 'basic' && state.basicAuthUsername && state.basicAuthPassword) {
            const credentials = btoa(`${state.basicAuthUsername}:${state.basicAuthPassword}`);
            headers['Authorization'] = `Basic ${credentials}`;
        } else if (state.authType === 'oidc' && state.oidcAccessToken) {
            headers['Authorization'] = `Bearer ${state.oidcAccessToken}`;
        }

        return createFetchClient<paths>({
            baseUrl: serverUrl.replace(/\/$/, ''),
            headers,
        });
    }

    /**
     * Prüft ob das Gerät online ist
     */
    async isOnline(): Promise<boolean> {
        try {
            const networkState = await Network.getNetworkStateAsync();
            return networkState.isConnected === true && networkState.isInternetReachable === true;
        } catch (e) {
            console.warn('Could not check network state:', e);
            return false;
        }
    }

    /**
     * Speichert den Watch-Progress lokal und queued für Sync
     */
    async saveWatchProgress(
        episode: components['schemas']['PodcastEpisodeDto'],
        watchedTimeMs: number,
        totalTimeMs: number
    ): Promise<void> {
        const progress: Omit<LocalWatchProgress, 'id'> = {
            episodeId: episode.episode_id,
            podcastId: episode.podcast_id,
            watchedTime: watchedTimeMs,
            totalTime: totalTimeMs,
            updatedAt: new Date().toISOString(),
            syncedAt: null,
            needsSync: true
        };

        await offlineDB.saveWatchProgress(progress);

        if (await this.isOnline()) {
            this.syncProgressToServer(episode.episode_id, watchedTimeMs).catch(console.error);
        }
    }

    /**
     * Synchronisiert einen einzelnen Progress-Eintrag mit dem Server
     */
    private async syncProgressToServer(episodeId: string, watchedTimeMs: number): Promise<boolean> {
        const client = this.getAuthenticatedClient();
        if (!client) return false;

        try {
            const { error } = await client.POST('/api/v1/podcasts/episode', {
                body: {
                    podcastEpisodeId: episodeId,
                    time: Math.floor(watchedTimeMs / 1000) // Server erwartet Sekunden
                }
            });

            if (!error) {
                await offlineDB.markProgressSynced(episodeId);
                return true;
            } else {
                console.warn(`Failed to sync progress for ${episodeId}:`, error);
                return false;
            }
        } catch (error) {
            console.error(`Error syncing progress for ${episodeId}:`, error);
            return false;
        }
    }

    /**
     * Synchronisiert alle unsynced Progress-Einträge
     */
    async syncAllProgress(): Promise<SyncResult> {
        if (this.isSyncing) {
            return { success: false, syncedItems: 0, failedItems: 0, errors: ['Sync already in progress'] };
        }

        if (!(await this.isOnline())) {
            return { success: false, syncedItems: 0, failedItems: 0, errors: ['Device is offline'] };
        }

        this.isSyncing = true;
        const result: SyncResult = {
            success: true,
            syncedItems: 0,
            failedItems: 0,
            errors: []
        };

        try {
            const unsyncedProgress = await offlineDB.getUnsyncedProgress();

            for (const progress of unsyncedProgress) {
                try {
                    const success = await this.syncProgressToServer(progress.episodeId, progress.watchedTime);
                    if (success) {
                        result.syncedItems++;
                    } else {
                        result.failedItems++;
                        result.errors.push(`Failed to sync ${progress.episodeId}`);
                    }
                } catch (error) {
                    result.failedItems++;
                    result.errors.push(`Error syncing ${progress.episodeId}: ${error}`);
                }
            }

            result.success = result.failedItems === 0;
        } catch (error) {
            result.success = false;
            result.errors.push(`Sync failed: ${error}`);
        } finally {
            this.isSyncing = false;
        }

        return result;
    }

    /**
     * Holt den Watch-Progress vom Server und merged mit lokalem Stand
     * @param episodeId Die Episode-ID
     * @param podcastId Optional: Die Podcast-ID (wird für neue Server-Daten benötigt)
     */
    async pullProgressFromServer(episodeId: string, podcastId?: number): Promise<LocalWatchProgress | null> {
        const client = this.getAuthenticatedClient();
        if (!client || !(await this.isOnline())) {
            return offlineDB.getWatchProgress(episodeId);
        }

        try {
            const { data: serverData, error } = await client.GET('/api/v1/podcasts/episode/{id}', {
                params: {
                    path: { id: episodeId }
                }
            });

            if (!error && serverData) {
                const localProgress = await offlineDB.getWatchProgress(episodeId);

                if (localProgress && localProgress.needsSync) {
                    return localProgress;
                }

                if (serverData.position !== undefined && serverData.position !== null) {
                    const resolvedPodcastId = podcastId ?? localProgress?.podcastId ?? 0;

                    const serverProgress: Omit<LocalWatchProgress, 'id'> = {
                        episodeId: episodeId,
                        podcastId: resolvedPodcastId,
                        watchedTime: serverData.position * 1000, // Server sendet Sekunden
                        totalTime: (serverData.total ?? 0) * 1000,
                        updatedAt: new Date().toISOString(),
                        syncedAt: new Date().toISOString(),
                        needsSync: false
                    };

                    await offlineDB.saveWatchProgress(serverProgress);
                    return { ...serverProgress, id: undefined };
                }
            }
        } catch (error) {
            console.error('Error pulling progress from server:', error);
        }

        // Fallback auf lokalen Stand
        return offlineDB.getWatchProgress(episodeId);
    }

    /**
     * Holt den lokalen Watch-Progress (ohne Server-Call)
     */
    async getLocalProgress(episodeId: string): Promise<LocalWatchProgress | null> {
        return offlineDB.getWatchProgress(episodeId);
    }

    /**
     * Startet die automatische Synchronisation
     */
    startAutoSync(intervalMs: number = 30000): void {
        this.stopAutoSync();

        // Initial sync
        this.syncAllProgress().catch(console.error);

        // Periodic sync
        this.syncInterval = setInterval(async () => {
            const online = await this.isOnline();

            // Notify listeners on state change
            if (this.lastOnlineState !== online) {
                this.lastOnlineState = online;
                this.listeners.forEach(listener => listener(online));

                // Sync when coming back online
                if (online) {
                    console.log('Device came online, syncing...');
                    this.syncAllProgress().catch(console.error);
                }
            }
        }, intervalMs);
    }

    /**
     * Stoppt die automatische Synchronisation
     */
    stopAutoSync(): void {
        if (this.syncInterval) {
            clearInterval(this.syncInterval);
            this.syncInterval = null;
        }
    }

    /**
     * Registriert einen Listener für Online-Status-Änderungen
     */
    subscribeToOnlineStatus(callback: (isOnline: boolean) => void): () => void {
        this.listeners.add(callback);

        this.isOnline().then(callback);

        return () => {
            this.listeners.delete(callback);
        };
    }

    /**
     * Holt die Anzahl der zu synchronisierenden Einträge
     */
    async getPendingSyncCount(): Promise<number> {
        const unsyncedProgress = await offlineDB.getUnsyncedProgress();
        return unsyncedProgress.length;
    }

    /**
     * Prüft ob gerade synchronisiert wird
     */
    isSyncInProgress(): boolean {
        return this.isSyncing;
    }
}

// Singleton-Instanz
export const syncService = new SyncServiceClass();
