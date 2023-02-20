import {createSlice} from '@reduxjs/toolkit'

export type Podcast = {
    directory: string,
    id: number,
    name: string,
    rssfeed: string
}

// Define a type for the slice state
interface CommonProps {
    sideBarCollapsed: boolean,
    podcasts:Podcast[]
}

// Define the initial state using that type
const initialState: CommonProps = {
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
        setPodcasts: (state, action) => {
            state.podcasts = action.payload
        }
}})

export const {setSideBarCollapsed, setPodcasts} = commonSlice.actions

export default commonSlice.reducer
