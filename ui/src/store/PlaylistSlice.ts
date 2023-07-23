import {createSlice, PayloadAction} from "@reduxjs/toolkit";
import {PlaylistDto} from "../models/Playlist";

interface PlaylistState {
    playlist: PlaylistDto[],
    createPlaylistOpen: boolean,
    currentPlaylistToEdit: PlaylistDto|undefined,
    selectedPlaylist: PlaylistDto|undefined
}

const initialState: PlaylistState = {
    playlist: [],
    createPlaylistOpen: false,
    selectedPlaylist: undefined,
    currentPlaylistToEdit: undefined
}
export const PlaylistSlice = createSlice({
    name: 'playlist',
    initialState,
    reducers:{
        setPlaylist: (state, action: PayloadAction<PlaylistDto[]>)=>{
            state.playlist = action.payload
        },
        setCreatePlaylistOpen: (state, action: PayloadAction<boolean>)=>{
            state.createPlaylistOpen = action.payload
        },
        setSelectedPlaylist: (state, action: PayloadAction<PlaylistDto>)=>{
            state.selectedPlaylist = action.payload
        },
        setCurrentPlaylistToEdit: (state, action:PayloadAction<PlaylistDto>)=>{
            state.currentPlaylistToEdit = action.payload
        }
    }
})

export const {setPlaylist, setCreatePlaylistOpen, setCurrentPlaylistToEdit,setSelectedPlaylist} = PlaylistSlice.actions
export default PlaylistSlice.reducer
