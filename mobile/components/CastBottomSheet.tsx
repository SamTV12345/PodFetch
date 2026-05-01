import { useCallback } from 'react';
import {
    Modal,
    Pressable,
    View,
    Text,
    ActivityIndicator,
    ScrollView,
    Alert,
} from 'react-native';
import { useTranslation } from 'react-i18next';
import { MaterialCommunityIcons, Ionicons, AntDesign } from '@expo/vector-icons';
import { styles } from '@/styles/styles';
import { $api } from '@/client';
import { useStore } from '@/store/store';
import { components } from '@/schema';
import { getAuthenticatedMediaUrl } from '@/utils/mediaUrl';

type CastDevice = components['schemas']['CastDeviceResponse'];

const isOnline = (agentId?: string | null, lastSeenAt?: string | null): boolean => {
    if (!agentId || !lastSeenAt) return false;
    const seen = Date.parse(lastSeenAt);
    if (Number.isNaN(seen)) return false;
    return Date.now() - seen < 60_000;
};

type Props = {
    visible: boolean;
    onClose: () => void;
};

export const CastBottomSheet = ({ visible, onClose }: Props) => {
    const { t } = useTranslation();
    const selectedPodcastEpisode = useStore((s) => s.podcastEpisodeRecord);
    const stopAndClearPlayer = useStore((s) => s.stopAndClearPlayer);
    const setCastSession = useStore((s) => s.setCastSession);
    const serverUrl = useStore((s) => s.serverUrl);
    const authType = useStore((s) => s.authType);
    const userApiKey = useStore((s) => s.userApiKey);
    const audioPlayer = useStore((s) => s.audioPlayer);

    const { data: devices, isLoading, refetch } = $api.useQuery(
        'get',
        '/api/v1/cast/devices',
        {},
        { enabled: visible },
    );

    const startSessionMutation = $api.useMutation('post', '/api/v1/cast/sessions');

    const handleSelectDevice = useCallback(
        async (device: CastDevice) => {
            if (!selectedPodcastEpisode?.podcastEpisode || !serverUrl) return;
            const ep = selectedPodcastEpisode.podcastEpisode;
            const apiKey = authType === 'basic' ? userApiKey : null;
            const playableUrl = ep.local_url
                ? getAuthenticatedMediaUrl(ep.local_url, serverUrl, apiKey)
                : ep.url;

            try {
                const session = await startSessionMutation.mutateAsync({
                    body: {
                        chromecast_uuid: device.chromecast_uuid,
                        episode_id: ep.episode_id,
                        url: playableUrl,
                        mime: 'audio/mpeg',
                        title: ep.name,
                        artwork_url: ep.image_url || null,
                        duration_secs: ep.total_time || null,
                    },
                });

                if (audioPlayer) {
                    try {
                        audioPlayer.pause();
                    } catch {
                        // ignore
                    }
                }
                stopAndClearPlayer();
                setCastSession(session, device.name);
                onClose();
            } catch (err) {
                console.error('Failed to start cast session:', err);
                Alert.alert(
                    t('error', { defaultValue: 'Error' }),
                    t('cast-start-failed', { defaultValue: 'Could not start casting.' }),
                );
            }
        },
        [
            selectedPodcastEpisode,
            serverUrl,
            authType,
            userApiKey,
            audioPlayer,
            stopAndClearPlayer,
            setCastSession,
            startSessionMutation,
            onClose,
            t,
        ],
    );

    return (
        <Modal
            animationType="slide"
            transparent
            visible={visible}
            onRequestClose={onClose}
        >
            <Pressable style={{ flex: 1, backgroundColor: 'rgba(0,0,0,0.5)' }} onPress={onClose}>
                <Pressable
                    style={{
                        position: 'absolute',
                        bottom: 0,
                        left: 0,
                        right: 0,
                        backgroundColor: styles.darkColor,
                        borderTopLeftRadius: 20,
                        borderTopRightRadius: 20,
                        maxHeight: '80%',
                        paddingBottom: 30,
                    }}
                    onPress={(e) => e.stopPropagation()}
                >
                    <View style={{
                        flexDirection: 'row',
                        alignItems: 'center',
                        padding: 18,
                        borderBottomWidth: 1,
                        borderBottomColor: styles.gray,
                    }}>
                        <MaterialCommunityIcons name="cast" size={22} color={styles.accentColor} />
                        <Text style={{
                            color: styles.white,
                            fontSize: 17,
                            fontWeight: '600',
                            marginLeft: 10,
                            flex: 1,
                        }}>
                            {t('cast-to', { defaultValue: 'Cast to' })}
                        </Text>
                        <Pressable onPress={onClose} hitSlop={10}>
                            <AntDesign name="close" size={22} color="white" />
                        </Pressable>
                    </View>

                    {isLoading ? (
                        <View style={{ padding: 40, alignItems: 'center' }}>
                            <ActivityIndicator size="large" color={styles.accentColor} />
                        </View>
                    ) : !devices || devices.length === 0 ? (
                        <View style={{ padding: 24 }}>
                            <Text style={{ color: styles.gray, fontSize: 14, textAlign: 'center' }}>
                                {t('cast-no-devices', {
                                    defaultValue: 'No cast devices available. An agent on your home LAN is required.',
                                })}
                            </Text>
                            <Pressable
                                onPress={() => refetch()}
                                style={{
                                    marginTop: 16,
                                    alignSelf: 'center',
                                    flexDirection: 'row',
                                    alignItems: 'center',
                                    gap: 6,
                                }}
                            >
                                <Ionicons name="refresh" size={16} color={styles.accentColor} />
                                <Text style={{ color: styles.accentColor, fontSize: 14 }}>
                                    {t('refresh', { defaultValue: 'Refresh' })}
                                </Text>
                            </Pressable>
                        </View>
                    ) : (
                        <ScrollView contentContainerStyle={{ paddingVertical: 8 }}>
                            {devices.map((device) => {
                                const online = isOnline(device.agent_id, device.last_seen_at);
                                const disabled = !online || startSessionMutation.isPending;
                                return (
                                    <Pressable
                                        key={device.chromecast_uuid}
                                        disabled={disabled}
                                        onPress={() => handleSelectDevice(device)}
                                        style={{
                                            flexDirection: 'row',
                                            alignItems: 'center',
                                            paddingVertical: 14,
                                            paddingHorizontal: 18,
                                            gap: 12,
                                            opacity: disabled ? 0.4 : 1,
                                        }}
                                    >
                                        <MaterialCommunityIcons
                                            name="cast"
                                            size={22}
                                            color={online ? styles.accentColor : styles.gray}
                                        />
                                        <View style={{ flex: 1 }}>
                                            <Text style={{ color: styles.white, fontSize: 15 }} numberOfLines={1}>
                                                {device.name}
                                            </Text>
                                            <Text style={{ color: styles.gray, fontSize: 12, marginTop: 2 }}>
                                                {online
                                                    ? t('online', { defaultValue: 'Online' })
                                                    : t('offline', { defaultValue: 'Offline' })}
                                                {' • '}
                                                {device.kind === 'chromecast_shared'
                                                    ? t('cast-kind-shared', { defaultValue: 'Shared' })
                                                    : t('cast-kind-personal', { defaultValue: 'Personal' })}
                                            </Text>
                                        </View>
                                        {startSessionMutation.isPending && (
                                            <ActivityIndicator size="small" color={styles.accentColor} />
                                        )}
                                    </Pressable>
                                );
                            })}
                        </ScrollView>
                    )}
                </Pressable>
            </Pressable>
        </Modal>
    );
};
