import { useSnackbar } from 'notistack'
import { useTranslation } from 'react-i18next'
import { $api } from '../utils/http'
import useCast from '../store/CastSlice'
import { components } from '../../schema'

type CastControlCommand = components['schemas']['CastControlCommand']

export type CastRemote = {
    isCasting: boolean
    sessionId?: string
    state?: components['schemas']['CastSessionState']
    positionSecs: number
    durationSecs: number
    deviceName: string
    pause: () => Promise<void>
    resume: () => Promise<void>
    seek: (positionSecs: number) => Promise<void>
    setVolume: (volume: number) => Promise<void>
    stop: () => Promise<void>
}

export const useCastRemote = (): CastRemote => {
    const { t } = useTranslation()
    const { enqueueSnackbar } = useSnackbar()
    const activeSession = useCast((s) => s.activeSession)
    const setActiveSession = useCast((s) => s.setActiveSession)
    const updateStatus = useCast((s) => s.updateStatus)
    const controlMutation = $api.useMutation('post', '/api/v1/cast/sessions/{id}/control')

    const send = async (cmd: CastControlCommand, optimistic?: () => void) => {
        if (!activeSession) return
        if (optimistic) optimistic()
        try {
            await controlMutation.mutateAsync({
                params: { path: { id: activeSession.sessionId } },
                body: cmd,
            })
        } catch {
            enqueueSnackbar(t('cast-control-not-available'), { variant: 'warning' })
        }
    }

    return {
        isCasting: !!activeSession,
        sessionId: activeSession?.sessionId,
        state: activeSession?.state,
        positionSecs: activeSession?.positionSecs ?? 0,
        durationSecs: activeSession?.durationSecs ?? 0,
        deviceName: activeSession?.deviceName ?? '',
        pause: () => send({ cmd: 'pause' }, () =>
            activeSession && updateStatus(activeSession.sessionId, { state: 'paused' })),
        resume: () => send({ cmd: 'resume' }, () =>
            activeSession && updateStatus(activeSession.sessionId, { state: 'playing' })),
        seek: (positionSecs: number) =>
            send({ cmd: 'seek', position_secs: positionSecs }, () =>
                activeSession && updateStatus(activeSession.sessionId, { positionSecs })),
        setVolume: (volume: number) =>
            send({ cmd: 'set_volume', volume }, () =>
                activeSession && updateStatus(activeSession.sessionId, { volume })),
        stop: async () => {
            await send({ cmd: 'stop' })
            setActiveSession(undefined)
        },
    }
}
