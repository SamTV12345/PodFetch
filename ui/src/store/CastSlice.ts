import { create } from "zustand";
import { components } from "../../schema";

export type CastSessionState = components["schemas"]["CastSessionState"];

export type ActiveCastSession = {
    sessionId: string;
    chromecastUuid: string;
    deviceName: string;
    episodeId?: number | null;
    state: CastSessionState;
    positionSecs: number;
    volume: number;
    durationSecs?: number;
};

type CastStore = {
    activeSession: ActiveCastSession | undefined;
    setActiveSession: (session: ActiveCastSession | undefined) => void;
    updateStatus: (
        sessionId: string,
        patch: Partial<Pick<ActiveCastSession, "state" | "positionSecs" | "volume">>,
    ) => void;
    clearIfMatches: (sessionId: string) => void;
};

const useCast = create<CastStore>()((set, get) => ({
    activeSession: undefined,
    setActiveSession: (activeSession) => set({ activeSession }),
    updateStatus: (sessionId, patch) => {
        const current = get().activeSession;
        if (!current || current.sessionId !== sessionId) return;
        set({ activeSession: { ...current, ...patch } });
    },
    clearIfMatches: (sessionId) => {
        const current = get().activeSession;
        if (current && current.sessionId === sessionId) {
            set({ activeSession: undefined });
        }
    },
}));

export default useCast;
