import { useStore } from '@/store/store';
import { AUDIO_PLAYER_HEIGHT } from '@/components/AudioPlayer';

/**
 * Hook, der den zusätzlichen Bottom-Padding zurückgibt, wenn der AudioPlayer aktiv ist.
 * Verhindert, dass Content vom Player verdeckt wird.
 */
export const useAudioPlayerPadding = () => {
    const podcastEpisodeRecord = useStore(state => state.podcastEpisodeRecord);
    const isPlayerActive = !!podcastEpisodeRecord;

    const baseTabPadding = 100;
    const playerPadding = isPlayerActive ? AUDIO_PLAYER_HEIGHT + 10 : 0;

    return {
        isPlayerActive,
        // Für ScrollViews mit contentContainerStyle
        contentPaddingBottom: baseTabPadding + playerPadding,
        // Für FlatLists
        listPaddingBottom: baseTabPadding + playerPadding,
        // Nur der zusätzliche Player-Padding
        playerPadding,
    };
};

