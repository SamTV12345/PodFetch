import {create} from "zustand";
import {PodFlix} from "../models/PodFlix";


type WatchTogetherState = {
    currentWatchTogetherCreate: WatchTogetherCreate|undefined,
    setWatchTogetherCreate: (currentWatchTogetherCreate: WatchTogetherCreate) => void,
    watchTogethers: PodFlix[],
    setWatchTogethers: (watchTogethers: PodFlix[]) => void
}

export const useWatchTogether = create<WatchTogetherState>((set, get) => ({
    currentWatchTogetherCreate: undefined,
    setWatchTogetherCreate: (currentWatchTogetherCreate: WatchTogetherCreate) => set({currentWatchTogetherCreate}),
    setWatchTogethers: (watchTogethers: PodFlix[]) => set({watchTogethers}),
    watchTogethers: []
}))
