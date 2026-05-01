import { useState, useCallback } from 'react';
import {
    View,
    TouchableOpacity,
    Alert,
    ActivityIndicator,
    ScrollView,
    Text,
} from 'react-native';
import { useTranslation } from 'react-i18next';
import { Ionicons, MaterialCommunityIcons } from '@expo/vector-icons';
import * as Clipboard from 'expo-clipboard';
import { ThemedText } from '@/components/ThemedText';
import { styles } from '@/styles/styles';
import { $api } from '@/client';
import { useStore } from '@/store/store';
import { useRouter } from 'expo-router';

type CastKind = 'chromecast_personal' | 'chromecast_shared';

const isOnline = (agentId?: string | null, lastSeenAt?: string | null): boolean => {
    if (!agentId) return false;
    if (!lastSeenAt) return false;
    const seen = Date.parse(lastSeenAt);
    if (Number.isNaN(seen)) return false;
    // Consider online if heartbeat within last 60 seconds
    return Date.now() - seen < 60_000;
};

const KindBadge = ({ kind }: { kind: CastKind }) => {
    const { t } = useTranslation();
    const isShared = kind === 'chromecast_shared';
    return (
        <View style={{
            paddingHorizontal: 8,
            paddingVertical: 3,
            borderRadius: 6,
            backgroundColor: isShared ? 'rgba(230,154,19,0.15)' : 'rgba(255,255,255,0.08)',
        }}>
            <Text style={{
                fontSize: 11,
                color: isShared ? styles.accentColor : styles.gray,
                fontWeight: '600',
            }}>
                {isShared
                    ? t('cast-kind-shared', { defaultValue: 'Shared' })
                    : t('cast-kind-personal', { defaultValue: 'Personal' })}
            </Text>
        </View>
    );
};

const CastDevicesScreen = () => {
    const { t } = useTranslation();
    const router = useRouter();
    const userProfile = useStore((s) => s.userProfile);
    const userApiKey = useStore((s) => s.userApiKey);
    const [isDiscovering, setIsDiscovering] = useState(false);

    const isAdmin = userProfile?.role === 'admin';

    const { data: devices, isLoading, refetch } = $api.useQuery(
        'get',
        '/api/v1/cast/devices',
        {},
    );

    const discoverMutation = $api.useMutation('post', '/api/v1/cast/devices/discover', {
        onSuccess: (data) => {
            setIsDiscovering(false);
            const count = data?.length ?? 0;
            Alert.alert(
                t('cast-discover-title', { defaultValue: 'Discovery complete' }),
                t('cast-discover-message', {
                    defaultValue: '{{count}} device(s) found on the LAN. Devices will appear here once an agent registers them.',
                    count,
                }),
            );
            refetch();
        },
        onError: () => {
            setIsDiscovering(false);
            Alert.alert(
                t('error', { defaultValue: 'Error' }),
                t('cast-discover-failed', { defaultValue: 'Discovery failed.' }),
            );
        },
    });

    const handleDiscover = useCallback(() => {
        setIsDiscovering(true);
        discoverMutation.mutate({});
    }, [discoverMutation]);

    const handleCopyApiKey = useCallback(async () => {
        if (userApiKey) {
            await Clipboard.setStringAsync(userApiKey);
            Alert.alert(
                t('copied', { defaultValue: 'Copied' }),
                t('api-key-copied', { defaultValue: 'API key copied.' }),
            );
        }
    }, [userApiKey, t]);

    return (
        <ScrollView
            style={{ flex: 1, backgroundColor: styles.lightDarkColor }}
            contentContainerStyle={{ paddingTop: 20, paddingHorizontal: 12, paddingBottom: 60 }}
        >
            {isAdmin && (
                <TouchableOpacity
                    onPress={handleDiscover}
                    disabled={isDiscovering}
                    style={{
                        backgroundColor: styles.accentColor,
                        borderRadius: 10,
                        padding: 14,
                        flexDirection: 'row',
                        alignItems: 'center',
                        justifyContent: 'center',
                        gap: 10,
                        marginBottom: 16,
                        opacity: isDiscovering ? 0.6 : 1,
                    }}
                >
                    {isDiscovering ? (
                        <ActivityIndicator size="small" color={styles.white} />
                    ) : (
                        <MaterialCommunityIcons name="magnify-scan" size={20} color={styles.white} />
                    )}
                    <ThemedText style={{ color: styles.white, fontWeight: '600' }}>
                        {t('cast-discover', { defaultValue: "Discover devices on this server's LAN" })}
                    </ThemedText>
                </TouchableOpacity>
            )}

            {isLoading ? (
                <View style={{ padding: 40, alignItems: 'center' }}>
                    <ActivityIndicator size="large" color={styles.accentColor} />
                </View>
            ) : !devices || devices.length === 0 ? (
                <View style={{
                    backgroundColor: styles.darkColor,
                    borderRadius: 10,
                    padding: 18,
                }}>
                    <View style={{ flexDirection: 'row', alignItems: 'center', gap: 10, marginBottom: 10 }}>
                        <MaterialCommunityIcons name="cast-off" size={22} color={styles.accentColor} />
                        <ThemedText style={{ color: styles.white, fontSize: 16, fontWeight: '600' }}>
                            {t('cast-empty-title', { defaultValue: 'No cast devices' })}
                        </ThemedText>
                    </View>
                    <ThemedText style={{ color: styles.gray, fontSize: 13, lineHeight: 19 }}>
                        {t('cast-empty-explainer', {
                            defaultValue:
                                'Casting requires a PodFetch agent running on the same LAN as your Chromecast. Run the CLI on a device at home with:',
                        })}
                    </ThemedText>
                    <View style={{
                        backgroundColor: 'rgba(0,0,0,0.4)',
                        borderRadius: 8,
                        padding: 10,
                        marginTop: 10,
                    }}>
                        <Text style={{ color: styles.white, fontFamily: 'monospace', fontSize: 12 }}>
                            podfetch --agent --remote URL --api-key YOUR_API_KEY
                        </Text>
                    </View>
                    <TouchableOpacity
                        onPress={() => {
                            if (userApiKey) {
                                handleCopyApiKey();
                            } else {
                                router.push('/(tabs)/settings');
                            }
                        }}
                        style={{
                            flexDirection: 'row',
                            alignItems: 'center',
                            gap: 6,
                            marginTop: 12,
                        }}
                    >
                        <Ionicons name="key-outline" size={16} color={styles.accentColor} />
                        <Text style={{ color: styles.accentColor, fontSize: 13 }}>
                            {userApiKey
                                ? t('cast-copy-api-key', { defaultValue: 'Copy your API key' })
                                : t('cast-find-api-key', { defaultValue: 'Find your API key in Settings' })}
                        </Text>
                    </TouchableOpacity>
                </View>
            ) : (
                <View style={{ gap: 10 }}>
                    {devices.map((device) => {
                        const online = isOnline(device.agent_id, device.last_seen_at);
                        return (
                            <View
                                key={device.chromecast_uuid}
                                style={{
                                    backgroundColor: styles.darkColor,
                                    borderRadius: 10,
                                    padding: 14,
                                }}
                            >
                                <View style={{ flexDirection: 'row', alignItems: 'center', gap: 10 }}>
                                    <MaterialCommunityIcons
                                        name="cast"
                                        size={24}
                                        color={online ? styles.accentColor : styles.gray}
                                    />
                                    <View style={{ flex: 1 }}>
                                        <ThemedText
                                            style={{ color: styles.white, fontSize: 15, fontWeight: '600' }}
                                            numberOfLines={1}
                                        >
                                            {device.name}
                                        </ThemedText>
                                        <View style={{ flexDirection: 'row', alignItems: 'center', gap: 8, marginTop: 4 }}>
                                            <KindBadge kind={device.kind} />
                                            <View style={{
                                                flexDirection: 'row',
                                                alignItems: 'center',
                                                gap: 4,
                                            }}>
                                                <View style={{
                                                    width: 6,
                                                    height: 6,
                                                    borderRadius: 3,
                                                    backgroundColor: online ? '#4caf50' : styles.gray,
                                                }} />
                                                <Text style={{ color: styles.gray, fontSize: 11 }}>
                                                    {online
                                                        ? t('online', { defaultValue: 'Online' })
                                                        : t('offline', { defaultValue: 'Offline' })}
                                                </Text>
                                            </View>
                                        </View>
                                    </View>
                                </View>
                            </View>
                        );
                    })}
                </View>
            )}
        </ScrollView>
    );
};

export default CastDevicesScreen;
