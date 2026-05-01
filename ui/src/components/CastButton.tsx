import { FC, useMemo } from 'react'
import * as Popover from '@radix-ui/react-popover'
import 'material-symbols/outlined.css'
import { useTranslation } from 'react-i18next'
import { useSnackbar } from 'notistack'
import useAudioPlayer from '../store/AudioPlayerSlice'
import useCast from '../store/CastSlice'
import { $api } from '../utils/http'
import { components } from '../../schema'
import { getAudioPlayer } from '../utils/audioPlayer'
import { cn } from '../lib/utils'

type CastDevice = components['schemas']['CastDeviceResponse']

const guessMimeFromUrl = (url: string | undefined | null): string => {
    if (!url) return 'audio/mpeg'
    const cleanPath = url.split('?')[0] || ''
    const ext = cleanPath.split('.').pop()?.toLowerCase() || ''
    switch (ext) {
        case 'm4a':
        case 'mp4':
        case 'aac':
            return 'audio/mp4'
        case 'ogg':
        case 'opus':
            return 'audio/ogg'
        case 'webm':
            return 'audio/webm'
        case 'wav':
            return 'audio/wav'
        case 'flac':
            return 'audio/flac'
        default:
            return 'audio/mpeg'
    }
}

export const CastButton: FC = () => {
    const { t } = useTranslation()
    const { enqueueSnackbar } = useSnackbar()
    const podcastEpisode = useAudioPlayer((state) => state.loadedPodcastEpisode)
    const activeSession = useCast((state) => state.activeSession)
    const setActiveSession = useCast((state) => state.setActiveSession)

    const devicesQuery = $api.useQuery('get', '/api/v1/cast/devices', {}, {
        // Devices change infrequently and are typically small.
        refetchOnWindowFocus: false,
    })
    const startSession = $api.useMutation('post', '/api/v1/cast/sessions')
    const controlSession = $api.useMutation('post', '/api/v1/cast/sessions/{id}/control')

    const devices: CastDevice[] = devicesQuery.data ?? []

    const activeDeviceName = useMemo(() => activeSession?.deviceName ?? '', [activeSession])

    const handlePickDevice = async (device: CastDevice) => {
        if (!podcastEpisode) {
            enqueueSnackbar(t('cast-no-episode-loaded'), { variant: 'warning' })
            return
        }
        const ep = podcastEpisode.podcastEpisode
        const url = ep.url || ep.local_url
        if (!url) {
            enqueueSnackbar(t('cast-no-stream-url'), { variant: 'error' })
            return
        }
        const audio = getAudioPlayer()
        if (audio && !audio.paused) {
            audio.pause()
        }
        try {
            const session = await startSession.mutateAsync({
                body: {
                    chromecast_uuid: device.chromecast_uuid,
                    episode_id: ep.id,
                    url,
                    mime: guessMimeFromUrl(url),
                    title: ep.name,
                    artwork_url: ep.image_url || ep.local_image_url || undefined,
                    duration_secs: ep.total_time > 0 ? ep.total_time : undefined,
                },
            })
            setActiveSession({
                sessionId: session.session_id,
                chromecastUuid: session.chromecast_uuid,
                deviceName: device.name,
                episodeId: session.episode_id ?? ep.id,
                state: session.state,
                positionSecs: session.position_secs,
                volume: session.volume,
                durationSecs: ep.total_time,
            })
        } catch {
            // Errors already surfaced via the response middleware snackbar.
        }
    }

    const handleStop = async () => {
        if (!activeSession) return
        try {
            await controlSession.mutateAsync({
                params: { path: { id: activeSession.sessionId } },
                body: { cmd: 'stop' },
            })
        } catch {
            // Server may return NotImplemented; clear the local session anyway.
        } finally {
            setActiveSession(undefined)
        }
    }

    const triggerLabel = activeSession
        ? t('casting-on', { name: activeDeviceName, defaultValue: `Casting on ${activeDeviceName}` })
        : t('cast')

    return (
        <Popover.Root>
            <Popover.Trigger asChild>
                <button
                    aria-label={triggerLabel}
                    title={triggerLabel}
                    className={cn(
                        'material-symbols-outlined cursor-pointer text-2xl ui-text hover:ui-text-hover',
                        activeSession ? 'ui-text-accent' : '',
                    )}
                >
                    {activeSession ? 'cast_connected' : 'cast'}
                </button>
            </Popover.Trigger>

            <Popover.Portal>
                <Popover.Content
                    sideOffset={6}
                    align="end"
                    className="ui-surface rounded-lg shadow-[0_4px_16px_rgba(0,0,0,var(--shadow-opacity))] z-50 min-w-[16rem] p-3"
                >
                    {activeSession ? (
                        <div className="flex flex-col gap-3">
                            <div className="text-sm ui-text">
                                {t('casting-on', { name: activeDeviceName, defaultValue: `Casting on ${activeDeviceName}` })}
                            </div>
                            <button
                                className="ui-bg-accent hover:ui-bg-accent-hover px-3 py-2 rounded-md text-sm ui-text-inverse"
                                onClick={handleStop}
                            >
                                {t('stop-casting')}
                            </button>
                        </div>
                    ) : devicesQuery.isLoading ? (
                        <div className="text-sm ui-text-muted px-2 py-1">{t('loading')}</div>
                    ) : devices.length === 0 ? (
                        <div className="text-sm ui-text-muted px-2 py-2">{t('cast-no-devices-visible')}</div>
                    ) : (
                        <ul className="flex flex-col">
                            {devices.map((device) => (
                                <li key={device.chromecast_uuid}>
                                    <button
                                        className="flex items-center gap-2 w-full text-left px-2 py-2 rounded-md text-sm ui-text hover:ui-text-hover hover:bg-(--surface-hover)"
                                        onClick={() => handlePickDevice(device)}
                                    >
                                        <span className="material-symbols-outlined">cast</span>
                                        <span className="grow truncate">{device.name}</span>
                                        <span className="text-xs ui-text-muted">
                                            {device.kind === 'chromecast_shared' ? t('cast-kind-shared') : t('cast-kind-personal')}
                                        </span>
                                    </button>
                                </li>
                            ))}
                        </ul>
                    )}
                    <Popover.Arrow className="ui-fill-inverse" />
                </Popover.Content>
            </Popover.Portal>
        </Popover.Root>
    )
}
