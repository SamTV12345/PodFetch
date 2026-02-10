import { View, TouchableOpacity, Alert, Switch } from 'react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
import { useTranslation } from 'react-i18next';
import { useRouter } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import { ThemedText } from '@/components/ThemedText';
import { ThemedView } from '@/components/ThemedView';
import { styles } from '@/styles/styles';
import { useStore } from '@/store/store';
import { IconSymbol } from '@/components/ui/IconSymbol';
import Heading1 from '@/components/text/Heading1';

const SettingsScreen = () => {
    const { t } = useTranslation();
    const router = useRouter();
    const serverUrl = useStore((state) => state.serverUrl);
    const clearServerUrl = useStore((state) => state.clearServerUrl);
    const offlineMode = useStore((state) => state.offlineMode);
    const toggleOfflineMode = useStore((state) => state.toggleOfflineMode);

    const handleDisconnect = () => {
        Alert.alert(
            t('disconnect-confirm-title'),
            t('disconnect-confirm-message'),
            [
                {
                    text: t('cancel'),
                    style: 'cancel',
                },
                {
                    text: t('disconnect'),
                    style: 'destructive',
                    onPress: () => {
                        clearServerUrl();
                        router.replace('/server-setup');
                    },
                },
            ]
        );
    };

    return (
        <SafeAreaView style={{ flex: 1, backgroundColor: styles.lightDarkColor }}>
            <ThemedView style={{
                flex: 1,
                backgroundColor: styles.lightDarkColor,
                paddingTop: 20,
                paddingHorizontal: 10,
            }}>
                <Heading1>{t('settings')}</Heading1>

                {/* Offline Mode Section */}
                <View style={{
                    backgroundColor: styles.darkColor,
                    borderRadius: 10,
                    padding: 15,
                    marginTop: 20,
                }}>
                    <View style={{
                        flexDirection: 'row',
                        alignItems: 'center',
                        justifyContent: 'space-between',
                    }}>
                        <View style={{
                            flexDirection: 'row',
                            alignItems: 'center',
                            gap: 12,
                            flex: 1,
                        }}>
                            <Ionicons
                                name={offlineMode ? "cloud-offline" : "cloud-done-outline"}
                                size={24}
                                color={offlineMode ? styles.accentColor : styles.gray}
                            />
                            <View style={{ flex: 1 }}>
                                <ThemedText style={{
                                    fontSize: 16,
                                    color: styles.white,
                                    fontWeight: '500'
                                }}>
                                    {t('offline-mode', { defaultValue: 'Offline-Modus' })}
                                </ThemedText>
                                <ThemedText style={{
                                    fontSize: 13,
                                    color: styles.gray,
                                    marginTop: 2,
                                }}>
                                    {t('offline-mode-description', {
                                        defaultValue: 'Nur heruntergeladene Inhalte anzeigen'
                                    })}
                                </ThemedText>
                            </View>
                        </View>
                        <Switch
                            value={offlineMode}
                            onValueChange={toggleOfflineMode}
                            trackColor={{
                                false: 'rgba(255,255,255,0.2)',
                                true: styles.accentColor
                            }}
                            thumbColor={offlineMode ? '#fff' : '#f4f3f4'}
                            ios_backgroundColor="rgba(255,255,255,0.2)"
                        />
                    </View>
                    {offlineMode && (
                        <View style={{
                            marginTop: 12,
                            paddingTop: 12,
                            borderTopWidth: 1,
                            borderTopColor: 'rgba(255,255,255,0.1)',
                            flexDirection: 'row',
                            alignItems: 'center',
                            gap: 8,
                        }}>
                            <Ionicons name="information-circle-outline" size={16} color={styles.accentColor} />
                            <ThemedText style={{
                                fontSize: 12,
                                color: styles.accentColor,
                                flex: 1,
                            }}>
                                {t('offline-mode-active-hint', {
                                    defaultValue: 'Du siehst nur heruntergeladene Episoden. Wiedergabe-Fortschritt wird lokal gespeichert und bei nächster Verbindung synchronisiert.'
                                })}
                            </ThemedText>
                        </View>
                    )}
                </View>

                {/* Server Info Section */}
                <View style={{
                    backgroundColor: styles.darkColor,
                    borderRadius: 10,
                    padding: 15,
                    marginTop: 20,
                }}>
                    <ThemedText style={{
                        fontSize: 14,
                        color: styles.gray,
                        marginBottom: 5
                    }}>
                        {t('connected-server')}
                    </ThemedText>
                    <ThemedText style={{
                        fontSize: 16,
                        color: styles.white,
                        fontWeight: '500'
                    }}>
                        {serverUrl || t('not-connected')}
                    </ThemedText>
                </View>

                {/* Disconnect Button */}
                <TouchableOpacity
                    style={{
                        backgroundColor: '#ff4444',
                        borderRadius: 10,
                        padding: 15,
                        marginTop: 20,
                        flexDirection: 'row',
                        alignItems: 'center',
                        justifyContent: 'center',
                        gap: 10,
                    }}
                    onPress={handleDisconnect}
                >
                    <IconSymbol name="power" size={20} color={styles.white} />
                    <ThemedText style={{
                        color: styles.white,
                        fontSize: 16,
                        fontWeight: '600'
                    }}>
                        {t('disconnect')}
                    </ThemedText>
                </TouchableOpacity>
            </ThemedView>
        </SafeAreaView>
    );
};

export default SettingsScreen;
