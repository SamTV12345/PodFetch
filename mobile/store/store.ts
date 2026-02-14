import { create } from 'zustand'
import {components} from "@/schema";
import { AudioPlayer } from 'expo-audio';
import {createJSONStorage, persist} from "zustand/middleware";
import * as SQLite from 'expo-sqlite';

export type AuthType = 'none' | 'basic' | 'oidc';

type ZustandStore = {
    podcastEpisodeRecord: components["schemas"]["PodcastWatchedEpisodeModelWithPodcastEpisode"]|undefined,
    setPodcastEpisodeRecord: (comp: components["schemas"]["PodcastWatchedEpisodeModelWithPodcastEpisode"])=>void,
    playEpisode: (episode: components["schemas"]["PodcastEpisodeDto"], podcast?: components["schemas"]["PodcastDto"], history?: components["schemas"]["EpisodeDto"] | null) => void,
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
    // Authentication
    authType: AuthType,
    setAuthType: (type: AuthType) => void,
    authToken: string | null,
    setAuthToken: (token: string | null) => void,
    // Basic Auth credentials
    basicAuthUsername: string | null,
    setBasicAuthUsername: (username: string | null) => void,
    basicAuthPassword: string | null,
    setBasicAuthPassword: (password: string | null) => void,
    // OIDC tokens
    oidcAccessToken: string | null,
    setOidcAccessToken: (token: string | null) => void,
    oidcRefreshToken: string | null,
    setOidcRefreshToken: (token: string | null) => void,
    oidcTokenExpiry: number | null,
    setOidcTokenExpiry: (expiry: number | null) => void,
    serverConfig: components["schemas"]["ConfigModel"] | null,
    setServerConfig: (config: components["schemas"]["ConfigModel"] | null) => void,
    clearAuth: () => void,
    // User profile
    userApiKey: string | null,
    setUserApiKey: (apiKey: string | null) => void,
    userProfile: components["schemas"]["UserWithAPiKey"] | null,
    setUserProfile: (profile: components["schemas"]["UserWithAPiKey"] | null) => void,
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
                    // Use position from episode object (in seconds) for seeking
                    const position = e.episode?.position ?? 0
                    audioPlayer.seekTo(position)
                    return
                }
                set({
                    podcastEpisodeRecord: e
                })
            },
            playEpisode: (episode, podcast, history) => {
                const oldPlayer = get().audioPlayer
                if (oldPlayer) {
                    try {
                        oldPlayer.pause()
                    } catch (e) {
                        console.warn('Error stopping old player:', e)
                    }
                }

                const record: components["schemas"]["PodcastWatchedEpisodeModelWithPodcastEpisode"] = {
                    podcastEpisode: episode,
                    podcast: podcast || {} as components["schemas"]["PodcastDto"],
                    episode: history || {
                        podcast: podcast?.rssfeed || '',
                        episode: episode.url,
                        timestamp: new Date().toISOString(),
                        guid: episode.guid,
                        action: 'play',
                        started: 0,
                        position: 0,
                        total: episode.total_time,
                        device: 'mobile',
                    },
                };
                set({ podcastEpisodeRecord: record, audioPlayer: undefined, isPlaying: false });
            },
            audioPlayer: undefined,
            setAudioPlayer: (player) => {
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
            // Authentication
            authType: 'none' as AuthType,
            setAuthType: (type) => set({ authType: type }),
            authToken: null,
            setAuthToken: (token) => set({ authToken: token }),
            // Basic Auth credentials
            basicAuthUsername: null,
            setBasicAuthUsername: (username) => set({ basicAuthUsername: username }),
            basicAuthPassword: null,
            setBasicAuthPassword: (password) => set({ basicAuthPassword: password }),
            // OIDC tokens
            oidcAccessToken: null,
            setOidcAccessToken: (token) => set({ oidcAccessToken: token }),
            oidcRefreshToken: null,
            setOidcRefreshToken: (token) => set({ oidcRefreshToken: token }),
            oidcTokenExpiry: null,
            setOidcTokenExpiry: (expiry) => set({ oidcTokenExpiry: expiry }),
            // Server config
            serverConfig: null,
            setServerConfig: (config) => set({ serverConfig: config }),
            clearAuth: () => set({
                authType: 'none',
                authToken: null,
                basicAuthUsername: null,
                basicAuthPassword: null,
                oidcAccessToken: null,
                oidcRefreshToken: null,
                oidcTokenExpiry: null,
                serverConfig: null,
                userApiKey: null,
                userProfile: null,
            }),
            // User profile
            userApiKey: null,
            setUserApiKey: (apiKey) => set({ userApiKey: apiKey }),
            userProfile: null,
            setUserProfile: (profile) => set({
                userProfile: profile,
                userApiKey: profile?.apiKey ?? null,
            }),
            // Offline mode
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
                authType: state.authType,
                basicAuthUsername: state.basicAuthUsername,
                basicAuthPassword: state.basicAuthPassword,
                oidcAccessToken: state.oidcAccessToken,
                oidcRefreshToken: state.oidcRefreshToken,
                oidcTokenExpiry: state.oidcTokenExpiry,
                serverConfig: state.serverConfig,
                userApiKey: state.userApiKey,
                userProfile: state.userProfile,
            }),
        },
    ),
)
