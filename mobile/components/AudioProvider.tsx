import { useAudioPlayer, useAudioPlayerStatus } from 'expo-audio';
import { useEffect, useMemo, useRef } from 'react';
import { useStore } from "@/store/store";

/**
 * Globaler Audio-Provider - wird nur einmal im Root-Layout gemountet.
 * Verwaltet den einzigen Audio-Player in der App.
 */
export const AudioProvider = ({ children }: { children: React.ReactNode }) => {
    const selectedPodcastEpisode = useStore(state => state.podcastEpisodeRecord);
    const setAudioPlayer = useStore(state => state.setAudioPlayer);
    const setAudioProgress = useStore(state => state.setAudioProgress);
    const setIsPlaying = useStore(state => state.setIsPlaying);

    // Track welche URL bereits gestartet wurde um Doppelstart zu verhindern
    const lastPlayedUrlRef = useRef<string | null>(null);
    const isInitializingRef = useRef(false);
    const hasRegisteredPlayerRef = useRef(false);

    const audioUrl = useMemo(() => {
        if (!selectedPodcastEpisode?.podcastEpisode.local_url) {
            return '';
        }
        return selectedPodcastEpisode.podcastEpisode.local_url;
    }, [selectedPodcastEpisode?.podcastEpisode.local_url]);

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

    // Update progress based on status
    useEffect(() => {
        if (status.duration > 0) {
            const progress = (status.currentTime / status.duration) * 100;
            setAudioProgress(progress);
        }
    }, [status.currentTime, status.duration, setAudioProgress]);

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
