import { create } from 'zustand'
import {components} from "@/schema";
import { AudioPlayer } from 'expo-audio';
import {createJSONStorage, persist} from "zustand/middleware";
import * as SQLite from 'expo-sqlite';


type ZustandStore = {
    podcastEpisodeRecord: components["schemas"]["PodcastWatchedEpisodeModelWithPodcastEpisode"]|undefined,
    setPodcastEpisodeRecord: (comp: components["schemas"]["PodcastWatchedEpisodeModelWithPodcastEpisode"])=>void,
    playEpisode: (episode: components["schemas"]["PodcastEpisodeDto"], podcast?: components["schemas"]["PodcastDto"]) => void,
    audioPlayer: AudioPlayer|undefined,
    setAudioPlayer: (player: AudioPlayer) => void,
    stopAndClearPlayer: () => void,
    isPlaying: boolean,
    setIsPlaying: (playing: boolean) => void,
    togglePlaying: ()=>void,
    audioProgress: number,
    setAudioProgress: (progress: number) => void,
    savePodcast: (id: string, podcast: components['schemas']['PodcastDto']) => void;
    // Server URL configuration
    serverUrl: string | null,
    setServerUrl: (url: string) => void,
    clearServerUrl: () => void,
    // Offline mode
    offlineMode: boolean,
    setOfflineMode: (enabled: boolean) => void,
    toggleOfflineMode: () => void,
}

const db =  SQLite.openDatabaseSync('podfetch.db')

db.execSync("CREATE TABLE IF NOT EXISTS storage (name TEXT PRIMARY KEY, value TEXT)")

class MVCStore {
    getItem(name: string) {
        const result = db.getFirstSync<{ value: string }>('SELECT value FROM storage WHERE name = ?', [name])
        return result?.value ?? null
    }
    setItem(name: string, value: string) {
        db.runSync('INSERT INTO storage (name, value) VALUES (?, ?) ON CONFLICT (name) DO UPDATE SET value = ?', [name, value, value])
    }
    removeItem(name: string) {
        db.runSync('DELETE FROM storage WHERE name = ?', [name])
    }
}


export const useStore = create<ZustandStore>()(
    persist(
        (set, get) => ({
            podcastEpisodeRecord: undefined,
            setPodcastEpisodeRecord: (e)=>{
                const podcastEpisodeRecord = get().podcastEpisodeRecord
                const audioPlayer = get().audioPlayer
                if (podcastEpisodeRecord && podcastEpisodeRecord.podcastEpisode.local_url == e.podcastEpisode.local_url && audioPlayer) {
                    audioPlayer.seekTo(e.watchedTime / 1000) // expo-audio uses seconds
                    return
                }
                set({
                    podcastEpisodeRecord: e
                })
                // Note: Audio player will be created and managed in the AudioPlayer component using useAudioPlayer hook
            },
            playEpisode: (episode, podcast) => {
                const oldPlayer = get().audioPlayer
                if (oldPlayer) {
                    try {
                        oldPlayer.pause()
                    } catch (e) {
                        console.warn('Error stopping old player:', e)
                    }
                }

                const record: components["schemas"]["PodcastWatchedEpisodeModelWithPodcastEpisode"] = {
                    date: new Date().toISOString(),
                    episodeId: episode.episode_id,
                    id: episode.id,
                    imageUrl: episode.image_url,
                    name: episode.name,
                    podcast: podcast || {} as components["schemas"]["PodcastDto"],
                    podcastEpisode: episode,
                    podcastId: episode.podcast_id,
                    totalTime: episode.total_time,
                    url: episode.url,
                    watchedTime: 0,
                };
                set({ podcastEpisodeRecord: record, audioPlayer: undefined, isPlaying: false });
            },
            audioPlayer: undefined,
            setAudioPlayer: (player) => {
                // Stoppe den alten Player bevor ein neuer gesetzt wird
                const oldPlayer = get().audioPlayer
                if (oldPlayer && oldPlayer !== player) {
                    try {
                        oldPlayer.pause()
                    } catch (e) {
                        console.warn('Error stopping old player:', e)
                    }
                }
                set({ audioPlayer: player })
            },
            stopAndClearPlayer: () => {
                const player = get().audioPlayer
                if (player) {
                    try {
                        player.pause()
                    } catch (e) {
                        console.warn('Error stopping player:', e)
                    }
                }
                set({ audioPlayer: undefined, isPlaying: false })
            },
            isPlaying: false,
            setIsPlaying: (playing) => set({ isPlaying: playing }),
            togglePlaying:()=>{
                const playing = !get().isPlaying
                const audioPlayer = get().audioPlayer
                set({isPlaying: playing})
                if (playing && audioPlayer) {
                    audioPlayer.play()
                } else if (!playing && audioPlayer) {
                    audioPlayer.pause()
                }
            },
            audioProgress: 0,
            setAudioProgress: (progress) => set({ audioProgress: progress }),
            savePodcast: (id, podcast)=>{
                set((state=>(
                    {
                        ...state,
                        [id]: podcast
                    }
                )))
            },
            serverUrl: null,
            setServerUrl: (url) => set({ serverUrl: url }),
            clearServerUrl: () => set({ serverUrl: null }),
            offlineMode: false,
            setOfflineMode: (enabled) => set({ offlineMode: enabled }),
            toggleOfflineMode: () => set((state) => ({ offlineMode: !state.offlineMode })),
        }),
        {
            name: 'podfetch-storage', // name of the item in the storage (must be unique)
            storage: createJSONStorage(() => new MVCStore()),
            // Only persist serializable fields
            partialize: (state) => ({
                serverUrl: state.serverUrl,
                offlineMode: state.offlineMode,
            }),
        },
    ),
)
