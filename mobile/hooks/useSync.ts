import { useState, useEffect, useCallback, useRef } from 'react';
import { syncService, SyncResult } from '@/store/syncService';
import { LocalWatchProgress } from '@/store/offlineStore';
import { components } from '@/schema';

/**
 * Hook für Watch-Progress mit automatischer Synchronisation
 */
export function useWatchProgress(episodeId: string | undefined) {
    const [progress, setProgress] = useState<LocalWatchProgress | null>(null);
    const [isLoading, setIsLoading] = useState<boolean>(true);
    const [isSyncing, setIsSyncing] = useState<boolean>(false);
    const lastSaveRef = useRef<number>(0);

    useEffect(() => {
        if (!episodeId) {
            setProgress(null);
            setIsLoading(false);
            return;
        }

        let mounted = true;

        const loadProgress = async () => {
            setIsLoading(true);
            const localProgress = await syncService.getLocalProgress(episodeId);
            if (mounted) {
                setProgress(localProgress);
                setIsLoading(false);
            }
        };

        loadProgress();

        return () => {
            mounted = false;
        };
    }, [episodeId]);

    /**
     * Speichert den Progress lokal und synchronisiert wenn möglich
     * Debounced um nicht zu viele Writes zu machen
     */
    const saveProgress = useCallback(async (
        episode: components['schemas']['PodcastEpisodeDto'],
        watchedTimeMs: number,
        totalTimeMs: number
    ) => {
        const now = Date.now();
        if (now - lastSaveRef.current < 5000) {
            return;
        }
        lastSaveRef.current = now;

        setIsSyncing(true);
        try {
            await syncService.saveWatchProgress(episode, watchedTimeMs, totalTimeMs);
            setProgress({
                episodeId: episode.episode_id,
                podcastId: episode.podcast_id,
                watchedTime: watchedTimeMs,
                totalTime: totalTimeMs,
                updatedAt: new Date().toISOString(),
                syncedAt: null,
                needsSync: true
            });
        } finally {
            setIsSyncing(false);
        }
    }, []);

    /**
     * Holt den Progress vom Server (wenn online) oder lokal
     */
    const pullProgress = useCallback(async () => {
        if (!episodeId) return null;

        setIsLoading(true);
        try {
            const serverProgress = await syncService.pullProgressFromServer(episodeId);
            setProgress(serverProgress);
            return serverProgress;
        } finally {
            setIsLoading(false);
        }
    }, [episodeId]);

    return {
        progress,
        isLoading,
        isSyncing,
        saveProgress,
        pullProgress,
        watchedTimeMs: progress?.watchedTime ?? 0,
        totalTimeMs: progress?.totalTime ?? 0,
        progressPercent: progress && progress.totalTime > 0
            ? (progress.watchedTime / progress.totalTime) * 100
            : 0
    };
}

/**
 * Hook für die Synchronisations-Steuerung
 */
export function useSync() {
    const [isSyncing, setIsSyncing] = useState<boolean>(false);
    const [pendingCount, setPendingCount] = useState<number>(0);
    const [lastResult, setLastResult] = useState<SyncResult | null>(null);

    // Initial load
    useEffect(() => {
        syncService.getPendingSyncCount().then(setPendingCount);
    }, []);

    const syncAll = useCallback(async () => {
        setIsSyncing(true);
        try {
            const result = await syncService.syncAllProgress();
            setLastResult(result);
            setPendingCount(await syncService.getPendingSyncCount());
            return result;
        } finally {
            setIsSyncing(false);
        }
    }, []);

    const refresh = useCallback(async () => {
        const count = await syncService.getPendingSyncCount();
        setPendingCount(count);
    }, []);

    return {
        isSyncing,
        pendingCount,
        lastResult,
        syncAll,
        refresh
    };
}

/**
 * Hook der automatische Synchronisation startet
 * Sollte im Root-Layout verwendet werden
 */
export function useAutoSync(intervalMs: number = 30000) {
    useEffect(() => {
        syncService.startAutoSync(intervalMs);

        return () => {
            syncService.stopAutoSync();
        };
    }, [intervalMs]);
}
