import { describe, it, expect, beforeEach } from "vitest"
import useCast, { ActiveCastSession } from "../src/store/CastSlice"

const sample: ActiveCastSession = {
    sessionId: "s1",
    chromecastUuid: "uuid-1",
    deviceName: "Living Room",
    episodeId: 42,
    state: "playing",
    positionSecs: 10,
    volume: 0.5,
    durationSecs: 1800,
}

describe("CastSlice store", () => {
    beforeEach(() => {
        useCast.getState().setActiveSession(undefined)
    })

    it("starts with no active session", () => {
        expect(useCast.getState().activeSession).toBeUndefined()
    })

    it("sets and reads the active session", () => {
        useCast.getState().setActiveSession(sample)
        expect(useCast.getState().activeSession?.deviceName).toBe("Living Room")
    })

    it("updates only when sessionId matches", () => {
        useCast.getState().setActiveSession(sample)
        useCast.getState().updateStatus("other", { positionSecs: 999 })
        expect(useCast.getState().activeSession?.positionSecs).toBe(10)

        useCast.getState().updateStatus("s1", { state: "paused", positionSecs: 50 })
        expect(useCast.getState().activeSession?.state).toBe("paused")
        expect(useCast.getState().activeSession?.positionSecs).toBe(50)
    })

    it("clears only when sessionId matches", () => {
        useCast.getState().setActiveSession(sample)
        useCast.getState().clearIfMatches("other")
        expect(useCast.getState().activeSession).toBeDefined()

        useCast.getState().clearIfMatches("s1")
        expect(useCast.getState().activeSession).toBeUndefined()
    })
})
