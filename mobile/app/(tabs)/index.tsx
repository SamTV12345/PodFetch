import {ThemedView} from '@/components/ThemedView';
import {SafeAreaView} from "react-native-safe-area-context";
import {styles} from "@/styles/styles";
import Heading1 from "@/components/text/Heading1";
import {$api, validatePodFetchServer} from "@/client";
import {useTranslation} from "react-i18next";
import {ScrollView, View, Text, Pressable, Image, RefreshControl} from "react-native";
import Heading2 from "@/components/text/Heading2";
import {LoadingSkeleton} from "@/components/ui/LoadingSkeleton";
import {PodcastCard} from "@/components/PodcastCard";
import {PodcastEpisodeCard} from "@/components/PodcastEpisodeCard";
import {useStore} from "@/store/store";
import {useRouter} from "expo-router";
import {Ionicons} from "@expo/vector-icons";
import {useEffect, useState, useCallback} from "react";
import {offlineDB, DownloadedEpisode} from "@/store/offlineStore";
import {TimelineEpisodeCard} from "@/components/TimelineEpisodeCard";

const HomeScreen = () => {
    const serverUrl = useStore((state) => state.serverUrl);
    const offlineMode = useStore((state) => state.offlineMode);
    const authType = useStore((state) => state.authType);
    const clearServerUrl = useStore((state) => state.clearServerUrl);
    const clearAuth = useStore((state) => state.clearAuth);
    const router = useRouter();
    const {t} = useTranslation();

    const [connectionError, setConnectionError] = useState<'auth-required' | 'connection-failed' | null>(null);

    const [offlineEpisodes, setOfflineEpisodes] = useState<DownloadedEpisode[]>([]);
    const [isLoadingOffline, setIsLoadingOffline] = useState(false);

    useEffect(() => {
        if (offlineMode) {
            setIsLoadingOffline(true);
            offlineDB.getAllDownloadedEpisodes()
                .then(episodes => setOfflineEpisodes(episodes))
                .catch(console.error)
                .finally(() => setIsLoadingOffline(false));
        }
    }, [offlineMode]);

    // Check server auth status on mount
    useEffect(() => {
        const checkServerAuth = async () => {
            if (!serverUrl || offlineMode) return;

            try {
                const result = await validatePodFetchServer(serverUrl);
                if (result.success) {
                    const config = result.config;
                    // Server requires auth but we have none configured
                    if ((config.basicAuth || config.oidcConfigured) && authType === 'none') {
                        setConnectionError('auth-required');
                    } else {
                        setConnectionError(null);
                    }
                } else {
                    setConnectionError('connection-failed');
                }
            } catch {
                setConnectionError('connection-failed');
            }
        };

        checkServerAuth();
    }, [serverUrl, offlineMode, authType]);

    const {data, isLoading, isError, error, refetch: refetchPodcasts} = $api.useQuery('get', '/api/v1/podcasts', {}, {
        enabled: !!serverUrl && !offlineMode && !connectionError,
    });
    const lastWatchedData = $api.useQuery('get', '/api/v1/podcasts/episode/lastwatched', {}, {
        enabled: !!serverUrl && !offlineMode && !connectionError,
    });

    const timelineData = $api.useQuery('get', '/api/v1/podcasts/timeline', {
        params: {
            query: {
                favoredOnly: false,
                notListened: false,
                favoredEpisodes: false,
            }
        }
    }, {
        enabled: !!serverUrl && !offlineMode && !connectionError,
    });

    const [isRefreshing, setIsRefreshing] = useState(false);

    const onRefresh = useCallback(async () => {
        if (offlineMode) return;

        setIsRefreshing(true);
        try {
            await Promise.all([
                refetchPodcasts(),
                lastWatchedData.refetch(),
                timelineData.refetch(),
            ]);
        } catch (e) {
            console.error('Refresh error:', e);
        } finally {
            setIsRefreshing(false);
        }
    }, [offlineMode, refetchPodcasts, lastWatchedData, timelineData]);

    // Check for API errors (e.g., 401 Unauthorized)
    useEffect(() => {
        if (isError && error) {
            // Check if it's an auth error
            const errorMessage = String(error);
            if (errorMessage.includes('401') || errorMessage.includes('Unauthorized') || errorMessage.includes('403')) {
                setConnectionError('auth-required');
            } else if (errorMessage.includes('Network') || errorMessage.includes('fetch')) {
                setConnectionError('connection-failed');
            }
        }
    }, [isError, error]);

    const handleReconnect = () => {
        // Clear auth and redirect to server setup
        clearAuth();
        clearServerUrl();
        router.replace('/server-setup');
    };

    // Connection Error Banner Component
    const ConnectionErrorBanner = () => {
        if (!connectionError) return null;

        const isAuthError = connectionError === 'auth-required';

        return (
            <Pressable onPress={handleReconnect}>
                <View style={{
                    flexDirection: 'row',
                    alignItems: 'center',
                    gap: 12,
                    backgroundColor: isAuthError ? 'rgba(255, 165, 0, 0.15)' : 'rgba(255, 68, 68, 0.15)',
                    paddingHorizontal: 16,
                    paddingVertical: 14,
                    borderRadius: 12,
                    marginBottom: 16,
                    borderWidth: 1,
                    borderColor: isAuthError ? 'rgba(255, 165, 0, 0.3)' : 'rgba(255, 68, 68, 0.3)',
                }}>
                    <Ionicons
                        name={isAuthError ? "lock-closed" : "cloud-offline"}
                        size={24}
                        color={isAuthError ? '#FFA500' : '#ff4444'}
                    />
                    <View style={{flex: 1}}>
                        <Text style={{
                            color: isAuthError ? '#FFA500' : '#ff4444',
                            fontSize: 15,
                            fontWeight: '600'
                        }}>
                            {isAuthError
                                ? t('auth-required-banner-title', { defaultValue: 'Anmeldung erforderlich' })
                                : t('connection-error-title', { defaultValue: 'Verbindungsfehler' })
                            }
                        </Text>
                        <Text style={{
                            color: isAuthError ? 'rgba(255, 165, 0, 0.8)' : 'rgba(255, 68, 68, 0.8)',
                            fontSize: 13,
                            marginTop: 2
                        }}>
                            {isAuthError
                                ? t('auth-required-banner-message', { defaultValue: 'Der Server erfordert jetzt eine Anmeldung. Tippe hier um dich anzumelden.' })
                                : t('connection-error-message', { defaultValue: 'Verbindung zum Server fehlgeschlagen. Tippe hier um es erneut zu versuchen.' })
                            }
                        </Text>
                    </View>
                    <Ionicons name="chevron-forward" size={20} color={isAuthError ? '#FFA500' : '#ff4444'} />
                </View>
            </Pressable>
        );
    };

    // Offline-Modus Ansicht
    if (offlineMode) {
        return (
            <SafeAreaView>
                <ThemedView style={{
                    backgroundColor: styles.lightDarkColor,
                    paddingTop: 20,
                    paddingLeft: 10,
                    paddingRight: 10,
                    minHeight: '100%',
                }}>
                    {/* Offline-Banner */}
                    <Pressable onPress={()=>{
                        router.push("/(tabs)/settings")
                    }}><View style={{
                        flexDirection: 'row',
                        alignItems: 'center',
                        gap: 8,
                        backgroundColor: 'rgba(255,255,255,0.1)',
                        paddingHorizontal: 16,
                        paddingVertical: 10,
                        borderRadius: 12,
                        marginBottom: 16,
                    }}>
                        <Ionicons name="cloud-offline" size={20} color={styles.accentColor} />
                        <Text style={{color: styles.accentColor, fontSize: 14, fontWeight: '500', flex: 1}}>
                            {t('offline-mode-active', { defaultValue: 'Offline-Modus aktiv' })}
                        </Text>
                    </View>
                    </Pressable>
                    <Heading1>{t('offline-downloads', { defaultValue: 'Deine Downloads' })}</Heading1>

                    {isLoadingOffline ? (
                        <ScrollView horizontal={true} style={{paddingBottom: 20}} overScrollMode="never">
                            <LoadingSkeleton/>
                            <LoadingSkeleton/>
                            <LoadingSkeleton/>
                        </ScrollView>
                    ) : offlineEpisodes.length > 0 ? (
                        <ScrollView style={{marginTop: 10}}>
                            <Text style={{color: styles.gray, marginBottom: 16}}>
                                {offlineEpisodes.length} {offlineEpisodes.length === 1 ? 'Episode' : 'Episoden'} verfügbar
                            </Text>
                            {offlineEpisodes.map(episode => (
                                <Pressable
                                    key={episode.episodeId}
                                    style={{
                                        flexDirection: 'row',
                                        alignItems: 'center',
                                        backgroundColor: 'rgba(255,255,255,0.05)',
                                        borderRadius: 12,
                                        padding: 12,
                                        marginBottom: 10,
                                    }}
                                    onPress={() => router.push(`/episodes/${episode.episodeId}`)}
                                >
                                    <Image
                                        source={{ uri: episode.imageUrl || episode.podcastImageUrl }}
                                        style={{
                                            width: 50,
                                            height: 50,
                                            borderRadius: 8,
                                            backgroundColor: 'rgba(255,255,255,0.1)',
                                            marginRight: 12,
                                        }}
                                    />
                                    <View style={{flex: 1}}>
                                        <Text style={{color: '#fff', fontWeight: '600', fontSize: 14}} numberOfLines={1}>
                                            {episode.name}
                                        </Text>
                                        <Text style={{color: styles.gray, fontSize: 12, marginTop: 2}} numberOfLines={1}>
                                            {episode.podcastName}
                                        </Text>
                                    </View>
                                    <Ionicons name="play-circle" size={32} color={styles.accentColor} />
                                </Pressable>
                            ))}
                        </ScrollView>
                    ) : (
                        <View style={{
                            alignItems: 'center',
                            justifyContent: 'center',
                            paddingVertical: 60,
                        }}>
                            <Ionicons name="cloud-download-outline" size={64} color="rgba(255,255,255,0.3)" />
                            <Text style={{color: '#fff', fontSize: 18, fontWeight: '600', marginTop: 16}}>
                                {t('no-downloads', { defaultValue: 'Keine Downloads' })}
                            </Text>
                            <Text style={{color: styles.gray, textAlign: 'center', marginTop: 8, paddingHorizontal: 40}}>
                                {t('no-downloads-hint', { defaultValue: 'Lade Episoden herunter, um sie offline zu hören.' })}
                            </Text>
                            <Pressable
                                style={{
                                    marginTop: 24,
                                    backgroundColor: styles.accentColor,
                                    paddingHorizontal: 24,
                                    paddingVertical: 12,
                                    borderRadius: 20,
                                }}
                                onPress={() => {
                                    useStore.getState().setOfflineMode(false);
                                }}
                            >
                                <Text style={{color: '#fff', fontWeight: '600'}}>
                                    {t('disable-offline-mode', { defaultValue: 'Online gehen' })}
                                </Text>
                            </Pressable>
                        </View>
                    )}
                </ThemedView>
            </SafeAreaView>
        );
    }

    // Online-Modus (bestehende Ansicht)
    return (
        <SafeAreaView style={{flex: 1}}>
            <ScrollView
                style={{flex: 1}}
                contentContainerStyle={{
                    backgroundColor: styles.lightDarkColor,
                    paddingTop: 20,
                    paddingLeft: 10,
                    paddingRight: 10,
                    paddingBottom: 120,
                }}
                refreshControl={
                    <RefreshControl
                        refreshing={isRefreshing}
                        onRefresh={onRefresh}
                        tintColor={styles.accentColor}
                        colors={[styles.accentColor]}
                        progressBackgroundColor={styles.darkColor}
                    />
                }
            >
                <Heading1>{t('home')}</Heading1>

                {/* Connection/Auth Error Banner */}
                <ConnectionErrorBanner />

                <Heading2>{t('last-listened')}</Heading2>
                <ScrollView horizontal={true} style={{paddingBottom: 20, display: 'flex', gap: 10}}
                            overScrollMode="never">
                    {lastWatchedData.isLoading &&
                        <>
                            <LoadingSkeleton/>
                            <LoadingSkeleton/>
                            <LoadingSkeleton/>
                            <LoadingSkeleton/>
                            <LoadingSkeleton/>
                            <LoadingSkeleton/>
                        </>
                    }
                    <View style={{display: 'flex', gap: 10, flexDirection: 'row'}}>
                        {
                            lastWatchedData.data && lastWatchedData.data.map(d => {
                                return <PodcastEpisodeCard podcastEpisode={d} key={"lastWatched"+d.podcastEpisode.episode_id}/>
                            })
                        }
                    </View>
                </ScrollView>

                {/* Neue Episoden aus der Timeline */}
                <Heading2>{t('new-episodes', { defaultValue: 'Neue Episoden' })}</Heading2>
                <ScrollView horizontal={true} style={{paddingBottom: 20, display: 'flex', gap: 10}}
                            overScrollMode="never">
                    {timelineData.isLoading &&
                        <>
                            <LoadingSkeleton/>
                            <LoadingSkeleton/>
                            <LoadingSkeleton/>
                            <LoadingSkeleton/>
                        </>
                    }
                    <View style={{display: 'flex', gap: 10, flexDirection: 'row'}}>
                        {
                            timelineData.data?.data && timelineData.data.data.slice(0, 10).map(d => {
                                return <TimelineEpisodeCard
                                    episode={d}
                                    key={"timeline"+d.podcast_episode.episode_id}
                                />
                            })
                        }
                    </View>
                </ScrollView>

                <Heading2 more onMore={() => {
                }}>{t('your-podcasts')}</Heading2>
                <ScrollView horizontal={true} style={{paddingBottom: 20, display: 'flex', gap: 10}}
                            overScrollMode="never">
                    {isLoading && <><LoadingSkeleton/>
                        <LoadingSkeleton/>
                        <LoadingSkeleton/>
                        <LoadingSkeleton/>
                        <LoadingSkeleton/>
                        <LoadingSkeleton/></>}
                    <View style={{display: 'flex', gap: 10, flexDirection: 'row'}}>
                        {
                            data && data.map(d => {
                                return <PodcastCard podcast={d} key={d.id}/>
                            })
                        }
                    </View>
                </ScrollView>
            </ScrollView>
        </SafeAreaView>
    );
}

export default HomeScreen;