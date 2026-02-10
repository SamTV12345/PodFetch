import * as SQLite from 'expo-sqlite';
import { components } from '@/schema';

/**
 * OfflineStore - Verwaltet lokale Persistenz für Offline-Funktionalität
 *
 * Speichert:
 * - Wiedergabe-Fortschritt (watchtime)
 * - Heruntergeladene Episoden-Metadaten
 * - Sync-Queue für spätere Server-Synchronisation
 */

// Typen
export interface LocalWatchProgress {
    id?: number;
    episodeId: string;
    podcastId: number;
    watchedTime: number;          // In Millisekunden
    totalTime: number;            // In Millisekunden
    updatedAt: string;            // ISO timestamp
    syncedAt: string | null;      // Wann zuletzt mit Server synchronisiert
    needsSync: boolean;           // Muss noch synchronisiert werden
}

export interface DownloadedEpisode {
    id?: number;
    episodeId: string;
    podcastId: number;
    name: string;
    localPath: string;            // Lokaler Dateipfad
    originalUrl: string;          // Original-URL vom Server
    imageUrl: string;
    totalTime: number;
    downloadedAt: string;         // ISO timestamp
    fileSize: number;             // Bytes
    podcastName: string;
    podcastImageUrl: string;
}

export interface SyncQueueItem {
    id?: number;
    type: 'watchtime' | 'download_complete';
    payload: string;              // JSON string
    createdAt: string;
    attempts: number;
    lastAttempt: string | null;
    error: string | null;
}

class OfflineDatabase {
    private db: SQLite.SQLiteDatabase;
    private initialized: boolean = false;

    constructor() {
        this.db = SQLite.openDatabaseSync('podfetch_offline.db');
    }

    async initialize(): Promise<void> {
        if (this.initialized) return;

        // Watchtime Progress Tabelle
        this.db.execSync(`
            CREATE TABLE IF NOT EXISTS watch_progress (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                episode_id TEXT UNIQUE NOT NULL,
                podcast_id INTEGER NOT NULL,
                watched_time INTEGER NOT NULL DEFAULT 0,
                total_time INTEGER NOT NULL DEFAULT 0,
                updated_at TEXT NOT NULL,
                synced_at TEXT,
                needs_sync INTEGER NOT NULL DEFAULT 1
            )
        `);

        // Downloaded Episodes Tabelle
        this.db.execSync(`
            CREATE TABLE IF NOT EXISTS downloaded_episodes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                episode_id TEXT UNIQUE NOT NULL,
                podcast_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                local_path TEXT NOT NULL,
                original_url TEXT NOT NULL,
                image_url TEXT,
                total_time INTEGER NOT NULL DEFAULT 0,
                downloaded_at TEXT NOT NULL,
                file_size INTEGER NOT NULL DEFAULT 0,
                podcast_name TEXT,
                podcast_image_url TEXT
            )
        `);

        // Sync Queue Tabelle
        this.db.execSync(`
            CREATE TABLE IF NOT EXISTS sync_queue (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                type TEXT NOT NULL,
                payload TEXT NOT NULL,
                created_at TEXT NOT NULL,
                attempts INTEGER NOT NULL DEFAULT 0,
                last_attempt TEXT,
                error TEXT
            )
        `);

        // Indizes für bessere Performance
        this.db.execSync(`
            CREATE INDEX IF NOT EXISTS idx_watch_progress_needs_sync 
            ON watch_progress(needs_sync)
        `);

        this.db.execSync(`
            CREATE INDEX IF NOT EXISTS idx_downloaded_episodes_podcast 
            ON downloaded_episodes(podcast_id)
        `);

        this.initialized = true;
    }

    // ==================== Watch Progress ====================

    async saveWatchProgress(progress: Omit<LocalWatchProgress, 'id'>): Promise<void> {
        await this.initialize();

        this.db.runSync(
            `INSERT INTO watch_progress 
                (episode_id, podcast_id, watched_time, total_time, updated_at, synced_at, needs_sync)
             VALUES (?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(episode_id) DO UPDATE SET
                watched_time = ?,
                total_time = ?,
                updated_at = ?,
                needs_sync = ?`,
            [
                progress.episodeId,
                progress.podcastId,
                progress.watchedTime,
                progress.totalTime,
                progress.updatedAt,
                progress.syncedAt,
                progress.needsSync ? 1 : 0,
                // Update values
                progress.watchedTime,
                progress.totalTime,
                progress.updatedAt,
                progress.needsSync ? 1 : 0
            ]
        );
    }

    async getWatchProgress(episodeId: string): Promise<LocalWatchProgress | null> {
        await this.initialize();

        const result = this.db.getFirstSync<{
            id: number;
            episode_id: string;
            podcast_id: number;
            watched_time: number;
            total_time: number;
            updated_at: string;
            synced_at: string | null;
            needs_sync: number;
        }>('SELECT * FROM watch_progress WHERE episode_id = ?', [episodeId]);

        if (!result) return null;

        return {
            id: result.id,
            episodeId: result.episode_id,
            podcastId: result.podcast_id,
            watchedTime: result.watched_time,
            totalTime: result.total_time,
            updatedAt: result.updated_at,
            syncedAt: result.synced_at,
            needsSync: result.needs_sync === 1
        };
    }

    async getUnsyncedProgress(): Promise<LocalWatchProgress[]> {
        await this.initialize();

        const results = this.db.getAllSync<{
            id: number;
            episode_id: string;
            podcast_id: number;
            watched_time: number;
            total_time: number;
            updated_at: string;
            synced_at: string | null;
            needs_sync: number;
        }>('SELECT * FROM watch_progress WHERE needs_sync = 1');

        return results.map(r => ({
            id: r.id,
            episodeId: r.episode_id,
            podcastId: r.podcast_id,
            watchedTime: r.watched_time,
            totalTime: r.total_time,
            updatedAt: r.updated_at,
            syncedAt: r.synced_at,
            needsSync: true
        }));
    }

    async markProgressSynced(episodeId: string): Promise<void> {
        await this.initialize();

        this.db.runSync(
            'UPDATE watch_progress SET needs_sync = 0, synced_at = ? WHERE episode_id = ?',
            [new Date().toISOString(), episodeId]
        );
    }

    // ==================== Downloaded Episodes ====================

    async saveDownloadedEpisode(episode: Omit<DownloadedEpisode, 'id'>): Promise<void> {
        await this.initialize();

        this.db.runSync(
            `INSERT INTO downloaded_episodes 
                (episode_id, podcast_id, name, local_path, original_url, image_url, 
                 total_time, downloaded_at, file_size, podcast_name, podcast_image_url)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(episode_id) DO UPDATE SET
                local_path = ?,
                downloaded_at = ?,
                file_size = ?`,
            [
                episode.episodeId,
                episode.podcastId,
                episode.name,
                episode.localPath,
                episode.originalUrl,
                episode.imageUrl,
                episode.totalTime,
                episode.downloadedAt,
                episode.fileSize,
                episode.podcastName,
                episode.podcastImageUrl,
                // Update values
                episode.localPath,
                episode.downloadedAt,
                episode.fileSize
            ]
        );
    }

    async getDownloadedEpisode(episodeId: string): Promise<DownloadedEpisode | null> {
        await this.initialize();

        const result = this.db.getFirstSync<{
            id: number;
            episode_id: string;
            podcast_id: number;
            name: string;
            local_path: string;
            original_url: string;
            image_url: string;
            total_time: number;
            downloaded_at: string;
            file_size: number;
            podcast_name: string;
            podcast_image_url: string;
        }>('SELECT * FROM downloaded_episodes WHERE episode_id = ?', [episodeId]);

        if (!result) return null;

        return {
            id: result.id,
            episodeId: result.episode_id,
            podcastId: result.podcast_id,
            name: result.name,
            localPath: result.local_path,
            originalUrl: result.original_url,
            imageUrl: result.image_url,
            totalTime: result.total_time,
            downloadedAt: result.downloaded_at,
            fileSize: result.file_size,
            podcastName: result.podcast_name,
            podcastImageUrl: result.podcast_image_url
        };
    }

    async getAllDownloadedEpisodes(): Promise<DownloadedEpisode[]> {
        await this.initialize();

        const results = this.db.getAllSync<{
            id: number;
            episode_id: string;
            podcast_id: number;
            name: string;
            local_path: string;
            original_url: string;
            image_url: string;
            total_time: number;
            downloaded_at: string;
            file_size: number;
            podcast_name: string;
            podcast_image_url: string;
        }>('SELECT * FROM downloaded_episodes ORDER BY downloaded_at DESC');

        return results.map(r => ({
            id: r.id,
            episodeId: r.episode_id,
            podcastId: r.podcast_id,
            name: r.name,
            localPath: r.local_path,
            originalUrl: r.original_url,
            imageUrl: r.image_url,
            totalTime: r.total_time,
            downloadedAt: r.downloaded_at,
            fileSize: r.file_size,
            podcastName: r.podcast_name,
            podcastImageUrl: r.podcast_image_url
        }));
    }

    async getDownloadedEpisodesByPodcast(podcastId: number): Promise<DownloadedEpisode[]> {
        await this.initialize();

        const results = this.db.getAllSync<{
            id: number;
            episode_id: string;
            podcast_id: number;
            name: string;
            local_path: string;
            original_url: string;
            image_url: string;
            total_time: number;
            downloaded_at: string;
            file_size: number;
            podcast_name: string;
            podcast_image_url: string;
        }>('SELECT * FROM downloaded_episodes WHERE podcast_id = ? ORDER BY downloaded_at DESC', [podcastId]);

        return results.map(r => ({
            id: r.id,
            episodeId: r.episode_id,
            podcastId: r.podcast_id,
            name: r.name,
            localPath: r.local_path,
            originalUrl: r.original_url,
            imageUrl: r.image_url,
            totalTime: r.total_time,
            downloadedAt: r.downloaded_at,
            fileSize: r.file_size,
            podcastName: r.podcast_name,
            podcastImageUrl: r.podcast_image_url
        }));
    }

    async deleteDownloadedEpisode(episodeId: string): Promise<void> {
        await this.initialize();

        this.db.runSync('DELETE FROM downloaded_episodes WHERE episode_id = ?', [episodeId]);
    }

    async isEpisodeDownloaded(episodeId: string): Promise<boolean> {
        await this.initialize();

        const result = this.db.getFirstSync<{ count: number }>(
            'SELECT COUNT(*) as count FROM downloaded_episodes WHERE episode_id = ?',
            [episodeId]
        );
        return (result?.count ?? 0) > 0;
    }

    // ==================== Sync Queue ====================

    async addToSyncQueue(item: Omit<SyncQueueItem, 'id' | 'attempts' | 'lastAttempt' | 'error'>): Promise<void> {
        await this.initialize();

        this.db.runSync(
            `INSERT INTO sync_queue (type, payload, created_at, attempts, last_attempt, error)
             VALUES (?, ?, ?, 0, NULL, NULL)`,
            [item.type, item.payload, item.createdAt]
        );
    }

    async getSyncQueue(): Promise<SyncQueueItem[]> {
        await this.initialize();

        const results = this.db.getAllSync<{
            id: number;
            type: string;
            payload: string;
            created_at: string;
            attempts: number;
            last_attempt: string | null;
            error: string | null;
        }>('SELECT * FROM sync_queue ORDER BY created_at ASC');

        return results.map(r => ({
            id: r.id,
            type: r.type as 'watchtime' | 'download_complete',
            payload: r.payload,
            createdAt: r.created_at,
            attempts: r.attempts,
            lastAttempt: r.last_attempt,
            error: r.error
        }));
    }

    async updateSyncQueueItem(id: number, success: boolean, error?: string): Promise<void> {
        await this.initialize();

        if (success) {
            this.db.runSync('DELETE FROM sync_queue WHERE id = ?', [id]);
        } else {
            this.db.runSync(
                'UPDATE sync_queue SET attempts = attempts + 1, last_attempt = ?, error = ? WHERE id = ?',
                [new Date().toISOString(), error ?? null, id]
            );
        }
    }

    async clearSyncQueue(): Promise<void> {
        await this.initialize();
        this.db.runSync('DELETE FROM sync_queue');
    }

    // ==================== Utility ====================

    async getTotalDownloadSize(): Promise<number> {
        await this.initialize();

        const result = this.db.getFirstSync<{ total: number }>(
            'SELECT COALESCE(SUM(file_size), 0) as total FROM downloaded_episodes'
        );
        return result?.total ?? 0;
    }

    async getDownloadCount(): Promise<number> {
        await this.initialize();

        const result = this.db.getFirstSync<{ count: number }>(
            'SELECT COUNT(*) as count FROM downloaded_episodes'
        );
        return result?.count ?? 0;
    }
}

// Singleton-Instanz
export const offlineDB = new OfflineDatabase();
