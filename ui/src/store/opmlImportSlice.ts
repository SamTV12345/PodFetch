import { create } from 'zustand'

interface OpmlImportProps {
	progress: boolean[]
	messages: string[]
	inProgress: boolean
	setProgress: (progress: boolean[]) => void
	setMessages: (messages: string[]) => void
	setInProgress: (inProgress: boolean) => void
}

const useOpmlImport = create<OpmlImportProps>((set) => ({
	progress: [],
	messages: [],
	inProgress: false,
	setProgress: (progress: boolean[]) => set({ progress }),
	setMessages: (messages: string[]) => set({ messages }),
	setInProgress: (inProgress: boolean) => set({ inProgress }),
}))

export default useOpmlImport
