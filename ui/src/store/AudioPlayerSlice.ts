import {createSlice, PayloadAction} from "@reduxjs/toolkit";
import {PodcastEpisode} from "./CommonSlice";

type AudioMetadata = {
    currentTime: number,
    duration: number,
    percentage: number
}

type AudioPlayerProps = {
    isPlaying: boolean,
    currentPodcast: PodcastEpisode|undefined,
    metadata: AudioMetadata|undefined,
}

const initialState: AudioPlayerProps = {
    isPlaying: false,
    currentPodcast: undefined,
    metadata: undefined
}

export const AudioPlayerSlice = createSlice({
    name: 'audioPlayer',
    initialState: initialState,
    reducers:{
        setPlaying: (state, action:PayloadAction<boolean>) => {
            state.isPlaying = action.payload
        },
        setCurrentPodcast: (state, action:PayloadAction<PodcastEpisode>) => {
            state.currentPodcast = action.payload
        },
        setMetadata: (state, action:PayloadAction<AudioMetadata>) => {
            state.metadata = action.payload
        },
        setCurrentTimeUpdate: (state, action:PayloadAction<number>) => {
            if(state.metadata){
                state.metadata.currentTime = action.payload
                state.metadata.percentage = (state.metadata.currentTime/state.metadata.duration)*100
            }
        },
        setCurrentTimeUpdatePercentage: (state, action:PayloadAction<number>) => {
            if(state.metadata){
                state.metadata.percentage = action.payload
                state.metadata.currentTime = (state.metadata.percentage/100)*state.metadata.duration
            }
        }
    }
})

export const {setPlaying, setCurrentPodcast, setMetadata, setCurrentTimeUpdate, setCurrentTimeUpdatePercentage} = AudioPlayerSlice.actions

export default AudioPlayerSlice.reducer
