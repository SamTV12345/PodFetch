import {PlaylistDto} from "../models/Playlist";
import {create} from "zustand";

interface PlaylistState {
    playlist: PlaylistDto[],
    createPlaylistOpen: boolean,
    currentPlaylistToEdit: PlaylistDto|undefined,
    selectedPlaylist: PlaylistDto|undefined,
    setPlaylist: (playlist: PlaylistDto[]) => void,
    setCreatePlaylistOpen: (createPlaylistOpen: boolean) => void,
    setCurrentPlaylistToEdit: (currentPlaylistToEdit: PlaylistDto) => void,
    setSelectedPlaylist: (selectedPlaylist: PlaylistDto) => void
}


const usePlaylist = create<PlaylistState>((set, get) => ({
    playlist: [],
    createPlaylistOpen: false,
    selectedPlaylist: undefined,
    currentPlaylistToEdit: undefined,
    setPlaylist: (playlist: PlaylistDto[]) => set({playlist}),
    setCreatePlaylistOpen: (createPlaylistOpen: boolean) => set({createPlaylistOpen}),
    setSelectedPlaylist: (selectedPlaylist: PlaylistDto) => set({selectedPlaylist}),
    setCurrentPlaylistToEdit: (currentPlaylistToEdit: PlaylistDto) => set({currentPlaylistToEdit})
}))

export default usePlaylist
