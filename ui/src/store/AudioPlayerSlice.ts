import {create} from "zustand";
import {components} from "../../schema";




type AudioMetadata = {
    currentTime: number,
    duration: number,
    percentage: number
}

type AudioPlayerProps = {
    isPlaying: boolean,
    currentPodcastEpisode: components["schemas"]["PodcastEpisodeWithHistory"]|undefined,
    currentPodcast: components["schemas"]["PodcastDto"]|undefined,
    metadata: AudioMetadata|undefined,
    volume: number,
    playBackRate: number,
    setPlaying: (isPlaying: boolean) => void,
    setCurrentPodcastEpisode: (currentPodcastEpisode: components["schemas"]["PodcastEpisodeWithHistory"]) => void,
    setMetadata: (metadata: AudioMetadata) => void,
    setCurrentTimeUpdate: (currentTime: number) => void,
    setCurrentTimeUpdatePercentage: (percentage: number) => void,
    setCurrentPodcast: (currentPodcast: components["schemas"]["PodcastDto"]) => void,
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
    setCurrentPodcastEpisode: (currentPodcastEpisode) => set({currentPodcastEpisode}),
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
    setCurrentPodcast: (currentPodcast: components["schemas"]["PodcastDto"]) => set({currentPodcast}),
    setVolume: (volume: number) => set({volume}),
    setPlayBackRate: (playBackRate: number) => set({playBackRate})
}))

export default useAudioPlayer
