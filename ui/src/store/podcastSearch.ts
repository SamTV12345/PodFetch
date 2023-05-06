import {createSlice} from "@reduxjs/toolkit";

const podcastSearchSlice = createSlice({
    name: 'podcastSearch',
    initialState: {
        searchText: "",
        orderOfPodcasts: "false",
        latestPub: false
    },
    reducers: {

    }
})

export const podcastSearch = podcastSearchSlice.reducer
export const podcastSearchActions = podcastSearchSlice.actions
