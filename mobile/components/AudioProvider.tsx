import { useAudioPlayer, useAudioPlayerStatus } from 'expo-audio';
import { useEffect, useMemo, useRef, useState } from 'react';
import { useStore } from "@/store/store";
import { syncService } from '@/store/syncService';
import { downloadManager } from '@/store/downloadManager';

/**
 * Globaler Audio-Provider - wird nur einmal im Root-Layout gemountet.
 * Verwaltet den einzigen Audio-Player in der App.
 * Speichert automatisch den Watch-Progress lokal und synchronisiert mit dem Server.
 */
export const AudioProvider = ({ children }: { children: React.ReactNode }) => {
    const selectedPodcastEpisode = useStore(state => state.podcastEpisodeRecord);
    const setAudioPlayer = useStore(state => state.setAudioPlayer);
    const setAudioProgress = useStore(state => state.setAudioProgress);
    const setIsPlaying = useStore(state => state.setIsPlaying);
    const serverUrl = useStore(state => state.serverUrl);

    // Track welche URL bereits gestartet wurde um Doppelstart zu verhindern
    const lastPlayedUrlRef = useRef<string | null>(null);
    const isInitializingRef = useRef(false);
    const hasRegisteredPlayerRef = useRef(false);
    const lastSaveTimeRef = useRef<number>(0);

    // Speichere effektive URL (lokal heruntergeladen oder Server)
    const [effectiveUrl, setEffectiveUrl] = useState<string>('');

    // Bestimme die beste verfügbare URL (lokal wenn heruntergeladen, sonst Server)
    useEffect(() => {
        const determineUrl = async () => {
            if (!selectedPodcastEpisode?.podcastEpisode) {
                setEffectiveUrl('');
                return;
            }

            const episode = selectedPodcastEpisode.podcastEpisode;

            // Prüfe ob Episode lokal heruntergeladen ist
            const localPath = await downloadManager.getLocalPath(episode.episode_id);
            if (localPath) {
                console.log('Using local downloaded file:', localPath);
                setEffectiveUrl(localPath);
                return;
            }

            // Sonst verwende Server-URL
            if (episode.local_url && serverUrl) {
                const fullUrl = serverUrl.replace(/\/$/, '') + episode.local_url;
                setEffectiveUrl(fullUrl);
            } else if (episode.url) {
                // Fallback auf Original-URL
                setEffectiveUrl(episode.url);
            } else {
                setEffectiveUrl('');
            }
        };

        determineUrl();
    }, [selectedPodcastEpisode?.podcastEpisode?.episode_id, selectedPodcastEpisode?.podcastEpisode?.local_url, serverUrl]);

    const audioUrl = effectiveUrl;

    const player = useAudioPlayer(audioUrl);
    const status = useAudioPlayerStatus(player);

    // Register player in store - nur einmal pro Player-Instanz
    useEffect(() => {
        if (player && audioUrl && !hasRegisteredPlayerRef.current) {
            hasRegisteredPlayerRef.current = true;
            setAudioPlayer(player);
        }

        // Reset wenn URL sich ändert
        if (!audioUrl) {
            hasRegisteredPlayerRef.current = false;
        }
    }, [player, audioUrl, setAudioPlayer]);

    // Update player reference when URL changes
    useEffect(() => {
        if (player && audioUrl) {
            hasRegisteredPlayerRef.current = false; // Reset für neue URL
            setAudioPlayer(player);
            hasRegisteredPlayerRef.current = true;
        }
    }, [audioUrl]);

    // Update progress based on status and save to offline store
    useEffect(() => {
        if (status.duration > 0) {
            const progress = (status.currentTime / status.duration) * 100;
            setAudioProgress(progress);

            // Speichere Progress alle 10 Sekunden lokal (und sync mit Server wenn online)
            const now = Date.now();
            if (now - lastSaveTimeRef.current >= 10000 && selectedPodcastEpisode?.podcastEpisode) {
                lastSaveTimeRef.current = now;

                // Speichere asynchron - blockiert nicht die UI
                syncService.saveWatchProgress(
                    selectedPodcastEpisode.podcastEpisode,
                    status.currentTime * 1000, // Konvertiere zu Millisekunden
                    status.duration * 1000
                ).catch(console.error);
            }
        }
    }, [status.currentTime, status.duration, setAudioProgress, selectedPodcastEpisode?.podcastEpisode]);

    // Auto-play when episode changes - aber nur einmal pro URL
    useEffect(() => {
        if (selectedPodcastEpisode && player && audioUrl &&
            audioUrl !== lastPlayedUrlRef.current &&
            !isInitializingRef.current) {

            isInitializingRef.current = true;
            lastPlayedUrlRef.current = audioUrl;

            // Kurze Verzögerung um sicherzustellen, dass der Player bereit ist
            const timeout = setTimeout(() => {
                player.play();
                setIsPlaying(true);
                isInitializingRef.current = false;
            }, 100);

            return () => {
                clearTimeout(timeout);
                isInitializingRef.current = false;
            };
        }
    }, [audioUrl, player, selectedPodcastEpisode, setIsPlaying]);

    return <>{children}</>;
};
