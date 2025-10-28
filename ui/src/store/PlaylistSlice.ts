import { create } from 'zustand'
import type { components } from '../../schema'

interface PlaylistState {
	playlist: components['schemas']['PlaylistDto'][]
	createPlaylistOpen: boolean
	currentPlaylistToEdit: components['schemas']['PlaylistDto'] | undefined
	selectedPlaylist: components['schemas']['PlaylistDto'] | undefined
	setPlaylist: (playlist: components['schemas']['PlaylistDto'][]) => void
	setCreatePlaylistOpen: (createPlaylistOpen: boolean) => void
	setCurrentPlaylistToEdit: (
		currentPlaylistToEdit: components['schemas']['PlaylistDto'],
	) => void
	setSelectedPlaylist: (
		selectedPlaylist: components['schemas']['PlaylistDto'],
	) => void
}

const usePlaylist = create<PlaylistState>((set, get) => ({
	playlist: [],
	createPlaylistOpen: false,
	selectedPlaylist: undefined,
	currentPlaylistToEdit: undefined,
	setPlaylist: (playlist: components['schemas']['PlaylistDto'][]) =>
		set({ playlist }),
	setCreatePlaylistOpen: (createPlaylistOpen: boolean) =>
		set({ createPlaylistOpen }),
	setSelectedPlaylist: (
		selectedPlaylist: components['schemas']['PlaylistDto'],
	) => set({ selectedPlaylist }),
	setCurrentPlaylistToEdit: (
		currentPlaylistToEdit: components['schemas']['PlaylistDto'],
	) => set({ currentPlaylistToEdit }),
}))

export default usePlaylist
