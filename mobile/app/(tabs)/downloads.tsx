import React from 'react';
import { View, Text, FlatList, Pressable, StyleSheet, Image, Alert } from 'react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
import { Ionicons } from '@expo/vector-icons';
import { useRouter } from 'expo-router';
import { useDownloadedEpisodes, formatBytes } from '@/hooks/useDownloads';
import { useNetworkStatus } from '@/hooks/useNetworkStatus';
import { useSync } from '@/hooks/useSync';
import { OfflineIndicator } from '@/components/OfflineIndicator';
import { DownloadedEpisode } from '@/store/offlineStore';
import { styles as appStyles } from '@/styles/styles';
import { useStore } from '@/store/store';
import { useAudioPlayerPadding } from '@/hooks/useAudioPlayerPadding';
import { useTranslation } from 'react-i18next';

export default function DownloadsScreen() {
    const { t } = useTranslation();
    const router = useRouter();
    const { episodes, isLoading, totalSize, deleteEpisode, clearAll, refresh } = useDownloadedEpisodes();
    const { isOnline } = useNetworkStatus();
    const { pendingCount, syncAll, isSyncing } = useSync();
    const offlineMode = useStore((state) => state.offlineMode);
    const { listPaddingBottom } = useAudioPlayerPadding();

    const handlePlayEpisode = (episode: DownloadedEpisode) => {
        router.push(`/episodes/${episode.episodeId}`);
    };

    const handleDeleteEpisode = (episode: DownloadedEpisode) => {
        Alert.alert(
            t('download-delete-title'),
            t('download-delete-episode-confirm', { name: episode.name }),
            [
                { text: t('cancel'), style: 'cancel' },
                {
                    text: t('delete'),
                    style: 'destructive',
                    onPress: () => deleteEpisode(episode.episodeId)
                }
            ]
        );
    };

    const handleClearAll = () => {
        if (episodes.length === 0) return;

        Alert.alert(
            t('downloads-delete-all-title'),
            t('downloads-delete-all-message', { count: episodes.length }),
            [
                { text: t('cancel'), style: 'cancel' },
                {
                    text: t('delete-all'),
                    style: 'destructive',
                    onPress: clearAll
                }
            ]
        );
    };

    const renderEpisode = ({ item }: { item: DownloadedEpisode }) => (
        <Pressable
            style={styles.episodeCard}
            onPress={() => handlePlayEpisode(item)}
        >
            <Image
                source={{ uri: item.imageUrl || item.podcastImageUrl }}
                style={styles.episodeImage}
            />
            <View style={styles.episodeInfo}>
                <Text style={styles.episodeName} numberOfLines={2}>
                    {item.name}
                </Text>
                <Text style={styles.podcastName} numberOfLines={1}>
                    {item.podcastName}
                </Text>
                <View style={styles.episodeMeta}>
                    <Ionicons name="download" size={12} color="#4ade80" />
                    <Text style={styles.metaText}>
                        {formatBytes(item.fileSize)}
                    </Text>
                    <Text style={styles.metaDivider}>•</Text>
                    <Text style={styles.metaText}>
                        {new Date(item.downloadedAt).toLocaleDateString('de-DE')}
                    </Text>
                </View>
            </View>
            <Pressable
                style={styles.deleteButton}
                onPress={() => handleDeleteEpisode(item)}
            >
                <Ionicons name="trash-outline" size={20} color="#ef4444" />
            </Pressable>
        </Pressable>
    );

    const renderHeader = () => (
        <View style={styles.header}>
            <View style={styles.headerInfo}>
                <Text style={styles.headerTitle}>{t('downloads')}</Text>
                <Text style={styles.headerSubtitle}>
                    {t('downloads-count', {
                        count: episodes.length,
                        itemLabel: episodes.length === 1 ? t('episode-singular') : t('episode-plural'),
                    })} • {formatBytes(totalSize)}
                </Text>
            </View>

            {episodes.length > 0 && (
                <Pressable onPress={handleClearAll} style={styles.clearButton}>
                    <Ionicons name="trash-bin-outline" size={20} color="#ef4444" />
                </Pressable>
            )}
        </View>
    );

    const renderEmpty = () => (
        <View style={styles.emptyContainer}>
            <Ionicons name="cloud-download-outline" size={64} color="rgba(255,255,255,0.3)" />
            <Text style={styles.emptyTitle}>{t('no-downloads')}</Text>
            <Text style={styles.emptySubtitle}>
                {t('no-downloads-hint')}
            </Text>
        </View>
    );

    return (
        <SafeAreaView style={styles.container} edges={['top']}>
            {renderHeader()}

            {/* Offline-Modus Banner */}
            {offlineMode && (
                <View style={styles.offlineModeBanner}>
                    <Ionicons name="cloud-offline" size={18} color={appStyles.accentColor} />
                    <Text style={styles.offlineModeBannerText}>
                        {t('offline-downloads-only')}
                    </Text>
                </View>
            )}

            <OfflineIndicator />

            {pendingCount > 0 && isOnline && (
                <Pressable
                    style={styles.syncBanner}
                    onPress={syncAll}
                    disabled={isSyncing}
                >
                    <Ionicons
                        name={isSyncing ? 'sync' : 'cloud-upload-outline'}
                        size={18}
                        color="#4ade80"
                    />
                    <Text style={styles.syncText}>
                        {isSyncing
                            ? t('syncing')
                            : t('sync-progress-items', { count: pendingCount })
                        }
                    </Text>
                </Pressable>
            )}

            <FlatList
                data={episodes}
                keyExtractor={(item) => item.episodeId}
                renderItem={renderEpisode}
                ListEmptyComponent={!isLoading ? renderEmpty : null}
                contentContainerStyle={episodes.length === 0 ? styles.emptyList : [styles.list, { paddingBottom: listPaddingBottom }]}
                onRefresh={refresh}
                refreshing={isLoading}
            />
        </SafeAreaView>
    );
}

const styles = StyleSheet.create({
    container: {
        flex: 1,
        backgroundColor: appStyles.lightDarkColor,
    },
    header: {
        flexDirection: 'row',
        justifyContent: 'space-between',
        alignItems: 'center',
        paddingHorizontal: 20,
        paddingVertical: 16,
    },
    headerInfo: {
        flex: 1,
    },
    headerTitle: {
        fontSize: 28,
        fontWeight: 'bold',
        color: '#fff',
    },
    headerSubtitle: {
        fontSize: 14,
        color: 'rgba(255,255,255,0.6)',
        marginTop: 4,
    },
    clearButton: {
        padding: 10,
        borderRadius: 20,
        backgroundColor: 'rgba(239, 68, 68, 0.1)',
    },
    syncBanner: {
        flexDirection: 'row',
        alignItems: 'center',
        justifyContent: 'center',
        gap: 8,
        marginHorizontal: 16,
        marginBottom: 8,
        paddingVertical: 10,
        paddingHorizontal: 16,
        backgroundColor: 'rgba(74, 222, 128, 0.1)',
        borderRadius: 8,
        borderWidth: 1,
        borderColor: 'rgba(74, 222, 128, 0.2)',
    },
    syncText: {
        color: '#4ade80',
        fontSize: 13,
        fontWeight: '500',
    },
    offlineModeBanner: {
        flexDirection: 'row',
        alignItems: 'center',
        gap: 8,
        marginHorizontal: 16,
        marginBottom: 8,
        paddingVertical: 10,
        paddingHorizontal: 16,
        backgroundColor: 'rgba(255,255,255,0.08)',
        borderRadius: 8,
        borderWidth: 1,
        borderColor: 'rgba(255,255,255,0.1)',
    },
    offlineModeBannerText: {
        color: appStyles.accentColor,
        fontSize: 13,
        fontWeight: '500',
        flex: 1,
    },
    list: {
        paddingHorizontal: 16,
        paddingBottom: 180, // Wird dynamisch überschrieben
    },
    emptyList: {
        flex: 1,
        justifyContent: 'center',
        alignItems: 'center',
    },
    episodeCard: {
        flexDirection: 'row',
        alignItems: 'center',
        backgroundColor: 'rgba(255,255,255,0.05)',
        borderRadius: 12,
        padding: 12,
        marginBottom: 10,
    },
    episodeImage: {
        width: 60,
        height: 60,
        borderRadius: 8,
        backgroundColor: 'rgba(255,255,255,0.1)',
    },
    episodeInfo: {
        flex: 1,
        marginLeft: 12,
        marginRight: 8,
    },
    episodeName: {
        fontSize: 15,
        fontWeight: '600',
        color: '#fff',
        lineHeight: 20,
    },
    podcastName: {
        fontSize: 13,
        color: 'rgba(255,255,255,0.6)',
        marginTop: 2,
    },
    episodeMeta: {
        flexDirection: 'row',
        alignItems: 'center',
        marginTop: 6,
        gap: 4,
    },
    metaText: {
        fontSize: 11,
        color: 'rgba(255,255,255,0.5)',
    },
    metaDivider: {
        color: 'rgba(255,255,255,0.3)',
        marginHorizontal: 2,
    },
    deleteButton: {
        padding: 8,
    },
    emptyContainer: {
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
        lineHeight: 20,
    },
});
