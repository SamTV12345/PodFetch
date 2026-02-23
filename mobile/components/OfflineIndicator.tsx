import React from 'react';
import { View, Text, StyleSheet, Pressable } from 'react-native';
import { Ionicons } from '@expo/vector-icons';
import { useNetworkStatus } from '@/hooks/useNetworkStatus';
import { useSync } from '@/hooks/useSync';
import { styles as appStyles } from '@/styles/styles';
import { useTranslation } from 'react-i18next';

interface OfflineIndicatorProps {
    showSyncButton?: boolean;
}

/**
 * Zeigt den Offline-Status und Sync-Informationen an
 */
export function OfflineIndicator({ showSyncButton = true }: OfflineIndicatorProps) {
    const { t } = useTranslation();
    const { isOnline } = useNetworkStatus();
    const { pendingCount, isSyncing, syncAll } = useSync();

    // Wenn online und nichts zu synchronisieren, zeige nichts
    if (isOnline && pendingCount === 0) {
        return null;
    }

    return (
        <View style={[
            styles.container,
            isOnline ? styles.onlineContainer : styles.offlineContainer
        ]}>
            <View style={styles.statusSection}>
                <Ionicons
                    name={isOnline ? 'cloud-outline' : 'cloud-offline-outline'}
                    size={16}
                    color={isOnline ? '#4ade80' : '#fbbf24'}
                />
                <Text style={styles.statusText}>
                    {isOnline ? t('online') : t('offline')}
                </Text>
            </View>

            {pendingCount > 0 && (
                <View style={styles.syncSection}>
                    <Text style={styles.pendingText}>
                        {t('sync-pending-items', {
                            count: pendingCount,
                            itemLabel: pendingCount === 1 ? t('sync-item-singular') : t('sync-item-plural'),
                        })}
                    </Text>

                    {showSyncButton && isOnline && (
                        <Pressable
                            onPress={syncAll}
                            disabled={isSyncing}
                            style={styles.syncButton}
                        >
                            <Ionicons
                                name={isSyncing ? 'sync' : 'sync-outline'}
                                size={16}
                                color="#fff"
                            />
                        </Pressable>
                    )}
                </View>
            )}
        </View>
    );
}

const styles = StyleSheet.create({
    container: {
        flexDirection: 'row',
        alignItems: 'center',
        justifyContent: 'space-between',
        paddingHorizontal: 16,
        paddingVertical: 8,
        borderRadius: 8,
        marginHorizontal: 16,
        marginVertical: 8,
    },
    offlineContainer: {
        backgroundColor: 'rgba(251, 191, 36, 0.15)',
        borderWidth: 1,
        borderColor: 'rgba(251, 191, 36, 0.3)',
    },
    onlineContainer: {
        backgroundColor: 'rgba(74, 222, 128, 0.1)',
        borderWidth: 1,
        borderColor: 'rgba(74, 222, 128, 0.2)',
    },
    statusSection: {
        flexDirection: 'row',
        alignItems: 'center',
        gap: 6,
    },
    statusText: {
        color: '#fff',
        fontSize: 13,
        fontWeight: '500',
    },
    syncSection: {
        flexDirection: 'row',
        alignItems: 'center',
        gap: 10,
    },
    pendingText: {
        color: 'rgba(255, 255, 255, 0.7)',
        fontSize: 12,
    },
    syncButton: {
        padding: 6,
        borderRadius: 20,
        backgroundColor: 'rgba(255, 255, 255, 0.1)',
    },
});
