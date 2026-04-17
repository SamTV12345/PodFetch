import {create} from "zustand";
import {components} from "../../schema";

interface PlaylistState {
    playlist: components["schemas"]["PlaylistDto"][],
    selectedPlaylist: components["schemas"]["PlaylistDto"]|undefined,
    setPlaylist: (playlist: components["schemas"]["PlaylistDto"][]) => void,
    setSelectedPlaylist: (selectedPlaylist: components["schemas"]["PlaylistDto"]) => void
}


const usePlaylist = create<PlaylistState>((set, get) => ({
    playlist: [],
    selectedPlaylist: undefined,
    setPlaylist: (playlist: components["schemas"]["PlaylistDto"][]) => set({playlist}),
    setSelectedPlaylist: (selectedPlaylist: components["schemas"]["PlaylistDto"]) => set({selectedPlaylist}),
}))

export default usePlaylist
