import { SafeAreaView } from "react-native-safe-area-context";
import { ScrollView, Text, View, Image, Pressable, StyleSheet, ActivityIndicator } from "react-native";
import MaterialIcons from "@expo/vector-icons/MaterialIcons";
import { Ionicons } from '@expo/vector-icons';
import FontAwesome from '@expo/vector-icons/FontAwesome';
import { router, useLocalSearchParams } from "expo-router";
import { useStore } from "@/store/store";
import { styles as appStyles } from "@/styles/styles";
import { useEffect, useState } from "react";
import { offlineDB, DownloadedEpisode } from "@/store/offlineStore";
import { formatBytes, useEpisodeDownload } from "@/hooks/useDownloads";
import { components } from "@/schema";
import { $api } from "@/client";
import { useTranslation } from "react-i18next";

// Hilfsfunktion: Sekunden zu mm:ss Format
const formatDuration = (seconds: number): string => {
    if (!seconds || seconds <= 0) return "0:00";
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
};

export default function EpisodeDetailScreen() {
    const { t } = useTranslation();
    const { id } = useLocalSearchParams<{ id: string }>();
    const offlineMode = useStore((state) => state.offlineMode);
    const serverUrl = useStore((state) => state.serverUrl);
    const playEpisode = useStore((state) => state.playEpisode);
    const currentEpisode = useStore((state) => state.podcastEpisodeRecord);
    const isPlaying = useStore((state) => state.isPlaying);
    const togglePlaying = useStore((state) => state.togglePlaying);

    const [offlineEpisode, setOfflineEpisode] = useState<DownloadedEpisode | null>(null);
    const [watchProgress, setWatchProgress] = useState<number>(0);
    const [isLoading, setIsLoading] = useState(true);

    // Online-Watchtime-Daten vom Server laden (nur wenn nicht im Offline-Modus)
    // Dieser Endpunkt gibt EpisodeDto zurück (action, position, total, etc.)
    const { data: onlineWatchData, isLoading: isOnlineLoading } = $api.useQuery(
        'get',
        '/api/v1/podcasts/episode/{id}',
        { params: { path: { id: id || '' } } },
        { enabled: !!id && !!serverUrl && !offlineMode }
    );

    // Download-Status prüfen
    const { isDownloaded } = useEpisodeDownload(id || '');

    // Lade Offline-Daten
    useEffect(() => {
        const loadOfflineData = async () => {
            if (!id) return;
            setIsLoading(true);
            try {
                const downloaded = await offlineDB.getDownloadedEpisode(id);
                setOfflineEpisode(downloaded);

                const progress = await offlineDB.getWatchProgress(id);
                if (progress) {
                    setWatchProgress(progress.watchedTime);
                }
            } catch (error) {
                console.error('Error loading offline data:', error);
            } finally {
                setIsLoading(false);
            }
        };
        loadOfflineData();
    }, [id]);

    // Watchtime vom Server (falls vorhanden) in lokale Daten mergen
    useEffect(() => {
        if (onlineWatchData && onlineWatchData.position) {
            // Server-Position in Millisekunden umrechnen (position ist in Sekunden)
            const serverProgress = onlineWatchData.position * 1000;
            if (serverProgress > watchProgress) {
                setWatchProgress(serverProgress);
            }
        }
    }, [onlineWatchData]);

    // Bestimme welche Daten angezeigt werden
    const isCurrentEpisode = currentEpisode?.podcastEpisode.episode_id === id;

    // Im Offline-Modus brauchen wir die Offline-Episode, im Online-Modus reicht sie auch
    // (da wir von Downloads hierher navigieren)
    const hasData = !!offlineEpisode;
    const combinedLoading = offlineMode ? isLoading : (isLoading && isOnlineLoading);

    const handlePlay = () => {
        if (isCurrentEpisode) {
            // Toggle play/pause für aktuelle Episode
            togglePlaying();
            return;
        }

        if (!id || !offlineEpisode) return;

        // Nutze Offline-Daten mit lokalem Pfad
        const offlineDto: components["schemas"]["PodcastEpisodeDto"] = {
            episode_id: offlineEpisode.episodeId,
            podcast_id: offlineEpisode.podcastId,
            name: offlineEpisode.name,
            url: offlineEpisode.localPath,
            local_url: offlineEpisode.localPath,
            image_url: offlineEpisode.imageUrl || offlineEpisode.podcastImageUrl,
            local_image_url: offlineEpisode.imageUrl,
            total_time: offlineEpisode.totalTime,
            description: '',
            date_of_recording: offlineEpisode.downloadedAt,
            deleted: false,
            episode_numbering_processed: true,
            guid: offlineEpisode.episodeId,
            id: offlineEpisode.id || 0,
            status: true,
        };

        // Erstelle History-Objekt mit gespeicherter Position
        const historyItem: components["schemas"]["EpisodeDto"] | undefined = onlineWatchData ? {
            ...onlineWatchData,
            // Position von Server oder lokalem Progress (in Sekunden)
            position: onlineWatchData.position ?? Math.floor(watchProgress / 1000),
        } : watchProgress > 0 ? {
            podcast: '',
            episode: offlineEpisode.localPath,
            timestamp: new Date().toISOString(),
            guid: offlineEpisode.episodeId,
            action: 'play',
            started: 0,
            position: Math.floor(watchProgress / 1000),
            total: offlineEpisode.totalTime,
            device: 'mobile',
        } : undefined;

        playEpisode(offlineDto, undefined, historyItem);
    };

    // Progress-Prozent berechnen
    const totalTime = offlineEpisode?.totalTime || onlineWatchData?.total || 0;
    const progressPercent = totalTime > 0
        ? Math.min((watchProgress / 1000) / totalTime * 100, 100)
        : 0;

    // Episoden-Daten für die Anzeige (primär aus Offline-Daten)
    const displayName = offlineEpisode?.name || '';
    const displayImage = offlineEpisode?.imageUrl || offlineEpisode?.podcastImageUrl || '';
    const displayPodcastName = offlineEpisode?.podcastName || '';
    const displayTotalTime = offlineEpisode?.totalTime || onlineWatchData?.total || 0;
    const displayFileSize = offlineEpisode?.fileSize;
    const displayDownloadedAt = offlineEpisode?.downloadedAt;

    if (combinedLoading) {
        return (
            <SafeAreaView style={styles.container}>
                <View style={styles.header}>
                    <MaterialIcons
                        size={32}
                        color="white"
                        name="chevron-left"
                        onPress={() => router.back()}
                    />
                </View>
                <View style={styles.loadingContainer}>
                    <ActivityIndicator size="large" color={appStyles.accentColor} />
                </View>
            </SafeAreaView>
        );
    }

    if (!hasData) {
        return (
            <SafeAreaView style={styles.container}>
                <View style={styles.header}>
                    <MaterialIcons
                        size={32}
                        color="white"
                        name="chevron-left"
                        onPress={() => router.back()}
                    />
                </View>
                <View style={styles.emptyContainer}>
                    <Ionicons name="alert-circle-outline" size={64} color="rgba(255,255,255,0.3)" />
                    <Text style={styles.emptyTitle}>
                        {offlineMode ? t('episode-not-available') : t('episode-not-found')}
                    </Text>
                    <Text style={styles.emptySubtitle}>
                        {offlineMode
                            ? t('episode-not-available-offline-message')
                            : t('episode-download-first-message')}
                    </Text>
                    <Pressable
                        style={styles.backButton}
                        onPress={() => router.back()}
                    >
                        <Text style={styles.backButtonText}>{t('back')}</Text>
                    </Pressable>
                </View>
            </SafeAreaView>
        );
    }

    return (
        <SafeAreaView style={styles.container}>
            <ScrollView overScrollMode="never" contentContainerStyle={styles.scrollContent}>
                {/* Header mit Zurück-Button */}
                <View style={styles.header}>
                    <MaterialIcons
                        size={32}
                        color="white"
                        name="chevron-left"
                        onPress={() => router.back()}
                    />
                    {offlineMode && (
                        <View style={styles.offlineBadge}>
                            <Ionicons name="cloud-offline" size={14} color="#fff" />
                            <Text style={styles.offlineBadgeText}>{t('offline')}</Text>
                        </View>
                    )}
                </View>

                {/* Episode-Bild */}
                <View style={styles.imageContainer}>
                    <Image
                        source={{ uri: displayImage }}
                        style={styles.episodeImage}
                    />
                    {isDownloaded && (
                        <View style={styles.downloadedBadge}>
                            <Ionicons name="checkmark-circle" size={20} color="#4ade80" />
                        </View>
                    )}
                </View>

                {/* Titel und Podcast-Name */}
                <Text style={styles.episodeTitle}>{displayName}</Text>
                {displayPodcastName && (
                    <Text style={styles.podcastName}>{displayPodcastName}</Text>
                )}

                {/* Meta-Informationen */}
                <View style={styles.metaContainer}>
                    <View style={styles.metaItem}>
                        <Ionicons name="time-outline" size={16} color={appStyles.gray} />
                        <Text style={styles.metaText}>
                            {formatDuration(displayTotalTime)}
                        </Text>
                    </View>
                    {displayDownloadedAt && (
                        <View style={styles.metaItem}>
                            <Ionicons name="download-outline" size={16} color={appStyles.gray} />
                            <Text style={styles.metaText}>
                                {new Date(displayDownloadedAt).toLocaleDateString('de-DE')}
                            </Text>
                        </View>
                    )}
                    {displayFileSize && (
                        <View style={styles.metaItem}>
                            <Ionicons name="document-outline" size={16} color="#4ade80" />
                            <Text style={[styles.metaText, { color: '#4ade80' }]}>
                                {formatBytes(displayFileSize)}
                            </Text>
                        </View>
                    )}
                </View>

                {/* Progress-Anzeige */}
                {progressPercent > 0 && (
                    <View style={styles.progressSection}>
                        <View style={styles.progressBar}>
                            <View style={[styles.progressFill, { width: `${progressPercent}%` }]} />
                        </View>
                        <Text style={styles.progressText}>
                            {formatDuration(watchProgress / 1000)} / {formatDuration(displayTotalTime)}
                        </Text>
                    </View>
                )}

                {/* Play-Button */}
                <Pressable
                    style={styles.playButton}
                    onPress={handlePlay}
                >
                    <FontAwesome
                        name={isCurrentEpisode && isPlaying ? "pause" : "play"}
                        size={24}
                        color="#fff"
                    />
                    <Text style={styles.playButtonText}>
                        {isCurrentEpisode && isPlaying
                            ? t('pause')
                            : progressPercent > 0
                                ? t('resume')
                                : t('play')}
                    </Text>
                </Pressable>

                {/* Hinweis-Box */}
                <View style={styles.offlineHint}>
                    <Ionicons name="information-circle-outline" size={20} color={appStyles.gray} />
                    <Text style={styles.offlineHintText}>
                        {t('offline-sync-hint')}
                    </Text>
                </View>
            </ScrollView>
        </SafeAreaView>
    );
}

const styles = StyleSheet.create({
    container: {
        flex: 1,
        backgroundColor: appStyles.lightDarkColor,
    },
    scrollContent: {
        paddingBottom: 120, // Platz für den Player
    },
    header: {
        flexDirection: 'row',
        justifyContent: 'space-between',
        alignItems: 'center',
        paddingHorizontal: 16,
        paddingVertical: 12,
    },
    offlineBadge: {
        flexDirection: 'row',
        alignItems: 'center',
        gap: 4,
        backgroundColor: 'rgba(255,255,255,0.1)',
        paddingHorizontal: 10,
        paddingVertical: 4,
        borderRadius: 12,
    },
    offlineBadgeText: {
        color: '#fff',
        fontSize: 12,
        fontWeight: '500',
    },
    loadingContainer: {
        flex: 1,
        justifyContent: 'center',
        alignItems: 'center',
    },
    emptyContainer: {
        flex: 1,
        justifyContent: 'center',
        alignItems: 'center',
        paddingHorizontal: 40,
    },
    emptyTitle: {
        fontSize: 20,
        fontWeight: '600',
        color: '#fff',
        marginTop: 16,
    },
    emptySubtitle: {
        fontSize: 14,
        color: 'rgba(255,255,255,0.5)',
        textAlign: 'center',
        marginTop: 8,
    },
    backButton: {
        marginTop: 24,
        backgroundColor: appStyles.accentColor,
        paddingHorizontal: 24,
        paddingVertical: 12,
        borderRadius: 20,
    },
    backButtonText: {
        color: '#fff',
        fontSize: 16,
        fontWeight: '600',
    },
    imageContainer: {
        alignItems: 'center',
        marginTop: 10,
        position: 'relative',
    },
    episodeImage: {
        width: 220,
        height: 220,
        borderRadius: 16,
        backgroundColor: 'rgba(255,255,255,0.1)',
    },
    downloadedBadge: {
        position: 'absolute',
        bottom: 8,
        right: '50%',
        marginRight: -110 + 8, // Rechte Ecke des Bildes
        backgroundColor: 'rgba(0,0,0,0.7)',
        borderRadius: 12,
        padding: 4,
    },
    episodeTitle: {
        fontSize: 22,
        fontWeight: 'bold',
        color: '#fff',
        textAlign: 'center',
        marginHorizontal: 20,
        marginTop: 20,
        lineHeight: 28,
    },
    podcastName: {
        fontSize: 15,
        color: appStyles.accentColor,
        textAlign: 'center',
        marginTop: 6,
    },
    metaContainer: {
        flexDirection: 'row',
        justifyContent: 'center',
        alignItems: 'center',
        gap: 20,
        marginTop: 16,
        flexWrap: 'wrap',
    },
    metaItem: {
        flexDirection: 'row',
        alignItems: 'center',
        gap: 4,
    },
    metaText: {
        fontSize: 13,
        color: appStyles.gray,
    },
    progressSection: {
        marginHorizontal: 20,
        marginTop: 24,
    },
    progressBar: {
        height: 4,
        backgroundColor: 'rgba(255,255,255,0.1)',
        borderRadius: 2,
        overflow: 'hidden',
    },
    progressFill: {
        height: '100%',
        backgroundColor: appStyles.accentColor,
        borderRadius: 2,
    },
    progressText: {
        fontSize: 12,
        color: appStyles.gray,
        textAlign: 'center',
        marginTop: 6,
    },
    playButton: {
        flexDirection: 'row',
        alignItems: 'center',
        justifyContent: 'center',
        gap: 12,
        backgroundColor: appStyles.accentColor,
        marginHorizontal: 20,
        marginTop: 24,
        paddingVertical: 16,
        borderRadius: 30,
    },
    playButtonText: {
        fontSize: 18,
        fontWeight: '600',
        color: '#fff',
    },
    offlineHint: {
        flexDirection: 'row',
        alignItems: 'center',
        gap: 8,
        marginHorizontal: 20,
        marginTop: 24,
        padding: 16,
        backgroundColor: 'rgba(255,255,255,0.05)',
        borderRadius: 12,
    },
    offlineHintText: {
        flex: 1,
        fontSize: 13,
        color: appStyles.gray,
        lineHeight: 18,
    },
});
