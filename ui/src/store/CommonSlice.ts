import {createSlice, PayloadAction} from '@reduxjs/toolkit'

export type Podcast = {
    directory: string,
    id: number,
    name: string,
    rssfeed: string,
    image_url: string,
}

export type PodcastEpisode = {
    id: number,
    podcast_id: number,
    episode_id: number,
    name: string,
    url: string,
    date: string,
    image_url: string,
}

// Define a type for the slice state
interface CommonProps {
    selectedEpisodes: PodcastEpisode[]
    sideBarCollapsed: boolean,
    podcasts:Podcast[]
}

// Define the initial state using that type
const initialState: CommonProps = {
    selectedEpisodes: [],
    sideBarCollapsed: false,
    podcasts: []
}

export const commonSlice = createSlice({
    name: 'commonSlice',
    // `createSlice` will infer the state type from the `initialState` argument
    initialState,
    reducers: {
        setSideBarCollapsed: (state, action) => {
            state.sideBarCollapsed = action.payload
        },
        setPodcasts: (state, action: PayloadAction<Podcast[]>) => {
            state.podcasts = action.payload
        },
        setSelectedEpisodes: (state, action:PayloadAction<PodcastEpisode[]>) => {
            state.selectedEpisodes = action.payload
        }
}})

export const {setSideBarCollapsed, setPodcasts,setSelectedEpisodes} = commonSlice.actions

export default commonSlice.reducer
