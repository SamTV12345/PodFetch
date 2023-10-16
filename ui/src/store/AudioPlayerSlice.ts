import {Podcast, PodcastEpisode} from "./CommonSlice";
import {create} from "zustand";

type AudioMetadata = {
    currentTime: number,
    duration: number,
    percentage: number
}

type AudioPlayerProps = {
    isPlaying: boolean,
    currentPodcastEpisode: PodcastEpisode|undefined,
    currentPodcast: Podcast|undefined,
    metadata: AudioMetadata|undefined,
    volume: number,
    playBackRate: number,
    setPlaying: (isPlaying: boolean) => void,
    setCurrentPodcastEpisode: (currentPodcastEpisode: PodcastEpisode) => void,
    setMetadata: (metadata: AudioMetadata) => void,
    setCurrentTimeUpdate: (currentTime: number) => void,
    setCurrentTimeUpdatePercentage: (percentage: number) => void,
    setCurrentPodcast: (currentPodcast: Podcast) => void,
    setVolume: (volume: number) => void,
    setPlayBackRate: (playBackRate: number) => void
}


const useAudioPlayer = create<AudioPlayerProps>()((set, get) => ({
    isPlaying: false,
    currentPodcastEpisode: undefined,
    currentPodcast: undefined,
    metadata: undefined,
    volume: 100,
    playBackRate: 1,
    setPlaying: (isPlaying: boolean) => set({isPlaying}),
    setCurrentPodcastEpisode: (currentPodcastEpisode: PodcastEpisode) => set({currentPodcastEpisode}),
    setMetadata: (metadata: AudioMetadata) => set({metadata}),
    setCurrentTimeUpdate: (currentTime: number) => {
        const metadata = get().metadata
        if(metadata){
                set({metadata: {...metadata, currentTime, percentage: (currentTime/metadata.duration)*100}})
        }
    },
    setCurrentTimeUpdatePercentage: (percentage: number) => {
        const metadata = get().metadata
        if(metadata){
            set({metadata: {...metadata, percentage, currentTime: (percentage/100)*metadata.duration}})
        }
    },
    setCurrentPodcast: (currentPodcast: Podcast) => set({currentPodcast}),
    setVolume: (volume: number) => set({volume}),
    setPlayBackRate: (playBackRate: number) => set({playBackRate})
}))

export default useAudioPlayer
