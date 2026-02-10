import {ThemedView} from '@/components/ThemedView';
import {SafeAreaView} from "react-native-safe-area-context";
import {styles} from "@/styles/styles";
import Heading1 from "@/components/text/Heading1";
import {$api} from "@/client";
import {useTranslation} from "react-i18next";
import {ScrollView, View, Text, Pressable, Image} from "react-native";
import Heading2 from "@/components/text/Heading2";
import {LoadingSkeleton} from "@/components/ui/LoadingSkeleton";
import {PodcastCard} from "@/components/PodcastCard";
import {PodcastEpisodeCard} from "@/components/PodcastEpisodeCard";
import {useStore} from "@/store/store";
import {useRouter} from "expo-router";
import {Ionicons} from "@expo/vector-icons";
import {useEffect, useState} from "react";
import {offlineDB, DownloadedEpisode} from "@/store/offlineStore";

const HomeScreen = () => {
    const serverUrl = useStore((state) => state.serverUrl);
    const offlineMode = useStore((state) => state.offlineMode);
    const router = useRouter();
    const {t} = useTranslation();

    // Offline-Downloads für den Offline-Modus
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

    // Online-Daten nur laden wenn nicht im Offline-Modus
    const {data, isLoading} = $api.useQuery('get', '/api/v1/podcasts', {}, {
        enabled: !!serverUrl && !offlineMode,
    });
    const lastWatchedData = $api.useQuery('get', '/api/v1/podcasts/episode/lastwatched', {}, {
        enabled: !!serverUrl && !offlineMode,
    });

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
        <SafeAreaView>
            <ThemedView style={{
                backgroundColor: styles.lightDarkColor,
                paddingTop: 20,
                paddingLeft: 10,
                paddingRight: 10
            }}>
                <Heading1>{t('home')}</Heading1>

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
                                return <PodcastEpisodeCard podcastEpisode={d} key={"lastWatched"+d.id}/>
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
            </ThemedView>
        </SafeAreaView>
    );
}

export default HomeScreen;