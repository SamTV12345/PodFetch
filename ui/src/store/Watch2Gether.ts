import {create} from "zustand";


type WatchTogetherState = {
    currentWatchTogetherCreate: WatchTogetherCreate|undefined,
    setWatchTogetherCreate: (currentWatchTogetherCreate: WatchTogetherCreate) => void,
}

export const useWatchTogether = create<WatchTogetherState>((set, get) => ({
    currentWatchTogetherCreate: undefined,
    setWatchTogetherCreate: (currentWatchTogetherCreate: WatchTogetherCreate) => set({currentWatchTogetherCreate})
}))
