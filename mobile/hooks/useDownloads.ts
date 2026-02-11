import { useState, useEffect, useCallback } from 'react';
import { downloadManager, DownloadProgress } from '@/store/downloadManager';
import { DownloadedEpisode } from '@/store/offlineStore';
import { components } from '@/schema';

/**
 * Hook für den Download-Status einer einzelnen Episode
 */
export function useEpisodeDownload(episodeId: string) {
    const [isDownloaded, setIsDownloaded] = useState<boolean>(false);
    const [isDownloading, setIsDownloading] = useState<boolean>(false);
    const [progress, setProgress] = useState<DownloadProgress | null>(null);
    const [localPath, setLocalPath] = useState<string | null>(null);
    const [isLoading, setIsLoading] = useState<boolean>(true);

    useEffect(() => {
        let mounted = true;

        const checkStatus = async () => {
            const downloaded = await downloadManager.isDownloaded(episodeId);
            if (mounted) {
                setIsDownloaded(downloaded);
                setIsDownloading(downloadManager.isDownloading(episodeId));

                if (downloaded) {
                    const path = await downloadManager.getLocalPath(episodeId);
                    setLocalPath(path);
                }
                setIsLoading(false);
            }
        };

        checkStatus();

        // Subscribe to progress updates
        const unsubscribe = downloadManager.subscribeToProgress(episodeId, (prog) => {
            if (mounted) {
                setProgress(prog);
                setIsDownloading(prog.status === 'downloading' || prog.status === 'pending');

                if (prog.status === 'completed') {
                    setIsDownloaded(true);
                    downloadManager.getLocalPath(episodeId).then(path => {
                        if (mounted) setLocalPath(path);
                    });
                } else if (prog.status === 'failed' || prog.status === 'cancelled') {
                    setIsDownloaded(false);
                    setLocalPath(null);
                }
            }
        });

        return () => {
            mounted = false;
            unsubscribe();
        };
    }, [episodeId]);

    const startDownload = useCallback(async (
        episode: components['schemas']['PodcastEpisodeDto'],
        podcast: components['schemas']['PodcastDto']
    ) => {
        setIsDownloading(true);
        try {
            await downloadManager.downloadEpisode(episode, podcast);
        } catch (error) {
            setIsDownloading(false);
            throw error;
        }
    }, []);

    const cancelDownload = useCallback(async () => {
        await downloadManager.cancelDownload(episodeId);
        setIsDownloading(false);
        setProgress(null);
    }, [episodeId]);

    const deleteDownload = useCallback(async () => {
        await downloadManager.deleteDownload(episodeId);
        setIsDownloaded(false);
        setLocalPath(null);
        setProgress(null);
    }, [episodeId]);

    const refresh = useCallback(async () => {
        setIsLoading(true);
        const downloaded = await downloadManager.isDownloaded(episodeId);
        setIsDownloaded(downloaded);
        setIsDownloading(downloadManager.isDownloading(episodeId));

        if (downloaded) {
            const path = await downloadManager.getLocalPath(episodeId);
            setLocalPath(path);
        } else {
            setLocalPath(null);
        }
        setIsLoading(false);
    }, [episodeId]);

    return {
        isDownloaded,
        isDownloading,
        progress,
        localPath,
        isLoading,
        startDownload,
        cancelDownload,
        deleteDownload,
        refresh
    };
}

/**
 * Hook für alle heruntergeladenen Episoden
 */
export function useDownloadedEpisodes() {
    const [episodes, setEpisodes] = useState<DownloadedEpisode[]>([]);
    const [isLoading, setIsLoading] = useState<boolean>(true);
    const [totalSize, setTotalSize] = useState<number>(0);

    const refresh = useCallback(async () => {
        setIsLoading(true);
        const [downloads, size] = await Promise.all([
            downloadManager.getAllDownloads(),
            downloadManager.getTotalDownloadSize()
        ]);
        setEpisodes(downloads);
        setTotalSize(size);
        setIsLoading(false);
    }, []);

    useEffect(() => {
        refresh();
    }, [refresh]);

    const deleteEpisode = useCallback(async (episodeId: string) => {
        await downloadManager.deleteDownload(episodeId);
        await refresh();
    }, [refresh]);

    const clearAll = useCallback(async () => {
        await downloadManager.clearAllDownloads();
        await refresh();
    }, [refresh]);

    return {
        episodes,
        isLoading,
        totalSize,
        refresh,
        deleteEpisode,
        clearAll
    };
}

/**
 * Hook für Downloads eines bestimmten Podcasts
 */
export function usePodcastDownloads(podcastId: number) {
    const [episodes, setEpisodes] = useState<DownloadedEpisode[]>([]);
    const [isLoading, setIsLoading] = useState<boolean>(true);

    const refresh = useCallback(async () => {
        setIsLoading(true);
        const downloads = await downloadManager.getDownloadsForPodcast(podcastId);
        setEpisodes(downloads);
        setIsLoading(false);
    }, [podcastId]);

    useEffect(() => {
        refresh();
    }, [refresh]);

    return {
        episodes,
        isLoading,
        refresh
    };
}

/**
 * Formatiert Bytes in menschenlesbare Größe
 */
export function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}
