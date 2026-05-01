import { useEffect } from 'react';
import { Alert } from 'react-native';
import { useTranslation } from 'react-i18next';
import { $api } from '@/client';
import { useStore } from '@/store/store';
import { components } from '@/schema';

type ControlBody = components['schemas']['CastControlRequest'];

/**
 * Poll the active cast session every 2 seconds.
 * Socket.io is not currently wired up in the mobile app, so polling is used for v1.
 */
export const useCastSessionPolling = () => {
    const castSession = useStore((s) => s.castSession);
    const setCastStatus = useStore((s) => s.setCastStatus);
    const clearCastSession = useStore((s) => s.clearCastSession);

    const sessionId = castSession?.session_id;

    const { data, error } = $api.useQuery(
        'get',
        '/api/v1/cast/sessions/{id}',
        sessionId
            ? { params: { path: { id: sessionId } } }
            : { params: { path: { id: '' } } },
        {
            enabled: !!sessionId,
            refetchInterval: 2000,
            refetchIntervalInBackground: false,
        },
    );

    useEffect(() => {
        if (data) {
            setCastStatus(data);
            if (data.state === 'stopped') {
                clearCastSession();
            }
        }
    }, [data, setCastStatus, clearCastSession]);

    useEffect(() => {
        if (error) {
            // Likely the session no longer exists on the server
            clearCastSession();
        }
    }, [error, clearCastSession]);
};

export const useCastControls = () => {
    const { t } = useTranslation();
    const castSession = useStore((s) => s.castSession);
    const clearCastSession = useStore((s) => s.clearCastSession);
    const controlMutation = $api.useMutation('post', '/api/v1/cast/sessions/{id}/control');

    const sendCommand = async (cmd: ControlBody): Promise<boolean> => {
        if (!castSession) return false;
        try {
            await controlMutation.mutateAsync({
                params: { path: { id: castSession.session_id } },
                body: cmd,
            });
            return true;
        } catch (err) {
            console.warn('Cast control failed:', err);
            Alert.alert(
                t('cast-control-unavailable-title', { defaultValue: 'Action not available' }),
                t('cast-control-unavailable-message', {
                    defaultValue: 'The agent does not support this command yet.',
                }),
            );
            return false;
        }
    };

    const stopCasting = async () => {
        if (!castSession) return;
        await sendCommand({ cmd: 'stop' });
        clearCastSession();
    };

    return {
        sendCommand,
        stopCasting,
        isPending: controlMutation.isPending,
    };
};
