import React from 'react';
import { View, Pressable, StyleSheet, ActivityIndicator, Alert, Text } from 'react-native';
import { Ionicons } from '@expo/vector-icons';
import { useEpisodeDownload } from '@/hooks/useDownloads';
import { components } from '@/schema';

interface DownloadButtonProps {
    episode: components['schemas']['PodcastEpisodeDto'];
    podcast: components['schemas']['PodcastDto'];
    size?: number;
    color?: string;
    showProgress?: boolean;
}

/**
 * Download-Button für Podcast-Episoden
 * Zeigt den Download-Status und ermöglicht Download/Löschen
 */
export function DownloadButton({
    episode,
    podcast,
    size = 24,
    color = '#fff',
    showProgress = true
}: DownloadButtonProps) {
    const {
        isDownloaded,
        isDownloading,
        progress,
        isLoading,
        startDownload,
        cancelDownload,
        deleteDownload
    } = useEpisodeDownload(episode.episode_id);

    if (isLoading) {
        return (
            <View style={[styles.button, { width: size + 16, height: size + 16 }]}>
                <ActivityIndicator size="small" color={color} />
            </View>
        );
    }

    const handlePress = async () => {
        if (isDownloading) {
            // Frage ob abbrechen
            Alert.alert(
                'Download abbrechen?',
                'Möchtest du den Download abbrechen?',
                [
                    { text: 'Nein', style: 'cancel' },
                    { text: 'Ja', onPress: cancelDownload, style: 'destructive' }
                ]
            );
        } else if (isDownloaded) {
            // Frage ob löschen
            Alert.alert(
                'Download löschen?',
                'Möchtest du die heruntergeladene Episode löschen?',
                [
                    { text: 'Nein', style: 'cancel' },
                    { text: 'Löschen', onPress: deleteDownload, style: 'destructive' }
                ]
            );
        } else {
            // Starte Download
            try {
                await startDownload(episode, podcast);
            } catch (error) {
                Alert.alert(
                    'Download fehlgeschlagen',
                    error instanceof Error ? error.message : 'Ein unbekannter Fehler ist aufgetreten'
                );
            }
        }
    };

    const getIconName = (): keyof typeof Ionicons.glyphMap => {
        if (isDownloading) return 'close-circle-outline';
        if (isDownloaded) return 'checkmark-circle';
        return 'download-outline';
    };

    const getIconColor = (): string => {
        if (isDownloaded) return '#4ade80'; // Grün für heruntergeladen
        if (isDownloading) return '#fbbf24'; // Gelb für laufend
        return color;
    };

    return (
        <Pressable
            onPress={handlePress}
            style={[styles.button, { width: size + 16, height: size + 16 }]}
        >
            {isDownloading && showProgress && progress ? (
                <View style={styles.progressContainer}>
                    <ActivityIndicator size="small" color="#4ade80" />
                    <Text style={styles.progressText}>
                        {Math.round(progress.progress * 100)}%
                    </Text>
                </View>
            ) : (
                <Ionicons
                    name={getIconName()}
                    size={size}
                    color={getIconColor()}
                />
            )}
        </Pressable>
    );
}

/**
 * Vereinfachter Download-Icon ohne interaktive Funktion
 * Zeigt nur den Status an
 */
export function DownloadStatusIcon({
    episodeId,
    size = 16,
    color = '#4ade80'
}: {
    episodeId: string;
    size?: number;
    color?: string;
}) {
    const { isDownloaded, isLoading } = useEpisodeDownload(episodeId);

    if (isLoading || !isDownloaded) {
        return null;
    }

    return (
        <Ionicons
            name="download"
            size={size}
            color={color}
        />
    );
}

const styles = StyleSheet.create({
    button: {
        justifyContent: 'center',
        alignItems: 'center',
        borderRadius: 100,
    },
    progressContainer: {
        justifyContent: 'center',
        alignItems: 'center',
    },
    progressText: {
        color: '#4ade80',
        fontSize: 10,
        fontWeight: '600',
        marginTop: 2,
    },
});
