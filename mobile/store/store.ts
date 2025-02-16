import { create } from 'zustand'
import {components} from "@/schema";
import {Audio} from 'expo-av'
import {SoundObject} from "expo-av/build/Audio/Sound";


type ZustandStore = {
    podcastEpisodeRecord: components["schemas"]["PodcastWatchedEpisodeModelWithPodcastEpisode"]|undefined,
    setPodcastEpisodeRecord: (comp: components["schemas"]["PodcastWatchedEpisodeModelWithPodcastEpisode"])=>void,
    audioSource: SoundObject|undefined,
    isPlaying: boolean,
    togglePlaying: ()=>void,
    audioProgress: number
}

export const useStore = create<ZustandStore>((set, get)=>({
    podcastEpisodeRecord: undefined,
    setPodcastEpisodeRecord: (e)=>{
        const podcastEpisodeRecord = get().podcastEpisodeRecord
        const audioSource = get().audioSource
        if (podcastEpisodeRecord && podcastEpisodeRecord.podcastEpisode.local_url == e.podcastEpisode.local_url && audioSource) {
            audioSource.sound.playAsync()
            return
        }
        set({
            podcastEpisodeRecord: e
        })
        Audio.Sound.createAsync({uri: e.podcastEpisode.local_url}, {shouldPlay: true}).then(audio=>{
            audio.sound!.setOnPlaybackStatusUpdate((status)=>{
                if (status.isLoaded) {
                    const status2 = status.positionMillis/status.durationMillis!
                    useStore.setState({
                        audioProgress: status2 * 100
                    })
                }
            })
            set({audioSource: audio})
            get().togglePlaying()
        }).catch(e=>{
            console.log(e)
        })
    },
    audioSource: undefined,
    isPlaying: false,
    togglePlaying:()=>{
        const playing = !get().isPlaying
        const audioSource = get().audioSource
        set({isPlaying: playing})
        if (playing && audioSource) {
            audioSource.sound.playAsync()
        } else if (!playing && audioSource) {
            audioSource.sound.pauseAsync()
        }
    },
    audioProgress: 0
}))
