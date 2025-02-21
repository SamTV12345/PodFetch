import { create } from 'zustand'
import {components} from "@/schema";
import {Audio} from 'expo-av'
import {SoundObject} from "expo-av/build/Audio/Sound";
import {createJSONStorage, persist} from "zustand/middleware";
import * as SQLite from 'expo-sqlite';


type ZustandStore = {
    podcastEpisodeRecord: components["schemas"]["PodcastWatchedEpisodeModelWithPodcastEpisode"]|undefined,
    setPodcastEpisodeRecord: (comp: components["schemas"]["PodcastWatchedEpisodeModelWithPodcastEpisode"])=>void,
    audioSource: SoundObject|undefined,
    isPlaying: boolean,
    togglePlaying: ()=>void,
    audioProgress: number,
    savePodcast: (id: string, podcast: components['schemas']['PodcastDto']) => void;
}

const db =  SQLite.openDatabaseSync('podfetch.db')

db.execSync("CREATE TABLE IF NOT EXISTS storage (name TEXT PRIMARY KEY, value TEXT)")

class MVCStore {
    getItem(name: string) {
        return db.getFirstSync('SELECT value FROM storage WHERE name = ?', [name]) as string || null
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
                const audioSource = get().audioSource
                if (podcastEpisodeRecord && podcastEpisodeRecord.podcastEpisode.local_url == e.podcastEpisode.local_url && audioSource) {
                    audioSource.sound.playFromPositionAsync(e.watchedTime)
                    return
                }
                set({
                    podcastEpisodeRecord: e
                })
                Audio.Sound.createAsync({uri: e.podcastEpisode.local_url}, {shouldPlay: true}).then(audio=>{
                    audio.sound!.setOnPlaybackStatusUpdate((status)=>{
                        if (status.isLoaded) {
                            const status2 = status.positionMillis/status.durationMillis!
                            useStore.setState({
                                audioProgress: status2 * 100
                            })
                        }
                    })
                    set({audioSource: audio})
                    get().togglePlaying()
                }).catch(e=>{
                    console.log(e)
                })
            },
            audioSource: undefined,
            isPlaying: false,
            togglePlaying:()=>{
                const playing = !get().isPlaying
                const audioSource = get().audioSource
                set({isPlaying: playing})
                if (playing && audioSource) {
                    audioSource.sound.playAsync()
                } else if (!playing && audioSource) {
                    audioSource.sound.pauseAsync()
                }
            },
            audioProgress: 0,
            savePodcast: (id, podcast)=>{
                set((state=>(
                    {
                        ...state,
                        [id]: podcast
                    }
                )))
            }
        }),
        {
            name: 'podfetch-storage', // name of the item in the storage (must be unique)
            storage: createJSONStorage(() => new MVCStore()),
        },
    ),
)
