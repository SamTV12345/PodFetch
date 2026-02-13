import { View, TouchableOpacity, Alert, Switch, Text, TextInput, ActivityIndicator, ScrollView } from 'react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
import { useTranslation } from 'react-i18next';
import { useRouter } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import * as Clipboard from 'expo-clipboard';
import { useState, useCallback } from 'react';
import { ThemedText } from '@/components/ThemedText';
import { styles } from '@/styles/styles';
import { useStore } from '@/store/store';
import { IconSymbol } from '@/components/ui/IconSymbol';
import Heading1 from '@/components/text/Heading1';
import { fetchUserProfile, updateUserProfile, generateNewApiKey } from "@/client";

const SettingsScreen = () => {
    const { t } = useTranslation();
    const router = useRouter();
    const serverUrl = useStore((state) => state.serverUrl);
    const clearServerUrl = useStore((state) => state.clearServerUrl);
    const clearAuth = useStore((state) => state.clearAuth);
    const offlineMode = useStore((state) => state.offlineMode);
    const toggleOfflineMode = useStore((state) => state.toggleOfflineMode);
    const authType = useStore((state) => state.authType);
    const userProfile = useStore((state) => state.userProfile);
    const userApiKey = useStore((state) => state.userApiKey);
    const setUserProfile = useStore((state) => state.setUserProfile);
    const basicAuthUsername = useStore((state) => state.basicAuthUsername);
    const basicAuthPassword = useStore((state) => state.basicAuthPassword);
    const setBasicAuthPassword = useStore((state) => state.setBasicAuthPassword);

    const [isRefreshing, setIsRefreshing] = useState(false);
    const [showPasswordChange, setShowPasswordChange] = useState(false);
    const [newPassword, setNewPassword] = useState('');
    const [confirmPassword, setConfirmPassword] = useState('');
    const [isUpdating, setIsUpdating] = useState(false);

    const handleRefreshProfile = useCallback(async () => {
        if (!serverUrl || !basicAuthUsername || !basicAuthPassword) return;

        setIsRefreshing(true);
        try {
            const profile = await fetchUserProfile(serverUrl, basicAuthUsername, basicAuthPassword);
            if (profile) {
                setUserProfile(profile);
                Alert.alert(
                    t('success', { defaultValue: 'Erfolg' }),
                    t('profile-refreshed', { defaultValue: 'Profil wurde aktualisiert.' })
                );
            } else {
                Alert.alert(
                    t('error', { defaultValue: 'Fehler' }),
                    t('profile-refresh-failed', { defaultValue: 'Profil konnte nicht aktualisiert werden.' })
                );
            }
        } finally {
            setIsRefreshing(false);
        }
    }, [serverUrl, basicAuthUsername, basicAuthPassword, setUserProfile, t]);

    const handleChangePassword = useCallback(async () => {
        if (!newPassword || !confirmPassword) {
            Alert.alert(
                t('error', { defaultValue: 'Fehler' }),
                t('password-required', { defaultValue: 'Bitte gib ein neues Passwort ein.' })
            );
            return;
        }

        if (newPassword !== confirmPassword) {
            Alert.alert(
                t('error', { defaultValue: 'Fehler' }),
                t('passwords-no-match', { defaultValue: 'Die Passwörter stimmen nicht überein.' })
            );
            return;
        }

        if (!serverUrl || !basicAuthUsername || !basicAuthPassword) return;

        setIsUpdating(true);
        try {
            const updatedProfile = await updateUserProfile(serverUrl, basicAuthUsername, basicAuthPassword, {
                username: basicAuthUsername,
                password: newPassword,
            });

            if (updatedProfile) {
                setUserProfile(updatedProfile);
                setBasicAuthPassword(newPassword);
                setNewPassword('');
                setConfirmPassword('');
                setShowPasswordChange(false);
                Alert.alert(
                    t('success', { defaultValue: 'Erfolg' }),
                    t('password-changed', { defaultValue: 'Passwort wurde geändert.' })
                );
            } else {
                Alert.alert(
                    t('error', { defaultValue: 'Fehler' }),
                    t('password-change-failed', { defaultValue: 'Passwort konnte nicht geändert werden.' })
                );
            }
        } finally {
            setIsUpdating(false);
        }
    }, [newPassword, confirmPassword, serverUrl, basicAuthUsername, basicAuthPassword, setUserProfile, setBasicAuthPassword, t]);

    const handleGenerateNewApiKey = useCallback(() => {
        Alert.alert(
            t('generate-api-key-title', { defaultValue: 'Neuen API-Key generieren?' }),
            t('generate-api-key-message', { defaultValue: 'Der alte API-Key wird ungültig. Möchtest du fortfahren?' }),
            [
                {
                    text: t('cancel'),
                    style: 'cancel',
                },
                {
                    text: t('generate', { defaultValue: 'Generieren' }),
                    onPress: async () => {
                        if (!serverUrl || !basicAuthUsername || !basicAuthPassword) return;

                        setIsUpdating(true);
                        try {
                            const newApiKey = generateNewApiKey();
                            const updatedProfile = await updateUserProfile(serverUrl, basicAuthUsername, basicAuthPassword, {
                                username: basicAuthUsername,
                                apiKey: newApiKey,
                            });

                            if (updatedProfile) {
                                setUserProfile(updatedProfile);
                                Alert.alert(
                                    t('success', { defaultValue: 'Erfolg' }),
                                    t('api-key-generated', { defaultValue: 'Neuer API-Key wurde generiert.' })
                                );
                            } else {
                                Alert.alert(
                                    t('error', { defaultValue: 'Fehler' }),
                                    t('api-key-generation-failed', { defaultValue: 'API-Key konnte nicht generiert werden.' })
                                );
                            }
                        } finally {
                            setIsUpdating(false);
                        }
                    },
                },
            ]
        );
    }, [serverUrl, basicAuthUsername, basicAuthPassword, setUserProfile, t]);

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
                        clearAuth();
                        clearServerUrl();
                        router.replace('/server-setup');
                    },
                },
            ]
        );
    };

    const handleCopyApiKey = async () => {
        if (userApiKey) {
            await Clipboard.setStringAsync(userApiKey);
            Alert.alert(
                t('copied', { defaultValue: 'Kopiert' }),
                t('api-key-copied', { defaultValue: 'API-Key wurde in die Zwischenablage kopiert.' })
            );
        }
    };

    return (
        <SafeAreaView style={{ flex: 1, backgroundColor: styles.lightDarkColor }}>
            <ScrollView style={{
                flex: 1,
                backgroundColor: styles.lightDarkColor,
            }} contentContainerStyle={{ paddingTop: 20, paddingHorizontal: 10, paddingBottom: 40 }}>
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


                {authType === 'basic' && userProfile && userProfile.role === 'admin' && (
                    <View style={{
                        backgroundColor: styles.darkColor,
                        borderRadius: 10,
                        padding: 15,
                        marginTop: 20,
                        opacity: offlineMode ? 0.6 : 1,
                    }}>
                        <View style={{
                            flexDirection: 'row',
                            alignItems: 'center',
                            gap: 12,
                            marginBottom: 15,
                        }}>
                            <Ionicons name="shield-checkmark" size={24} color={offlineMode ? styles.gray : styles.accentColor} />
                            <ThemedText style={{
                                fontSize: 18,
                                color: styles.white,
                                fontWeight: '600'
                            }}>
                                {t('admin-functions', { defaultValue: 'Administrator-Funktionen' })}
                            </ThemedText>
                        </View>

                        {/* Offline Mode Warning */}
                        {offlineMode && (
                            <View style={{
                                backgroundColor: 'rgba(255,152,0,0.1)',
                                borderLeftWidth: 3,
                                borderLeftColor: '#ff9800',
                                padding: 12,
                                borderRadius: 8,
                                marginBottom: 12,
                                flexDirection: 'row',
                                alignItems: 'center',
                                gap: 10,
                            }}>
                                <Ionicons name="warning" size={20} color="#ff9800" />
                                <ThemedText style={{
                                    fontSize: 13,
                                    color: '#ff9800',
                                    flex: 1,
                                }}>
                                    {t('admin-offline-warning', {
                                        defaultValue: 'Im Offline-Modus sind Admin-Funktionen nicht verfügbar. Bitte verbinde dich mit dem Server.'
                                    })}
                                </ThemedText>
                            </View>
                        )}

                        {/* Add Podcast */}
                        <TouchableOpacity
                            style={{
                                flexDirection: 'row',
                                alignItems: 'center',
                                gap: 12,
                                paddingVertical: 12,
                                borderTopWidth: 1,
                                borderTopColor: 'rgba(255,255,255,0.1)',
                            }}
                            onPress={() => router.push('/add-podcast')}
                            disabled={offlineMode}
                        >
                            <Ionicons name="add-circle" size={20} color={offlineMode ? 'rgba(255,255,255,0.3)' : styles.gray} />
                            <View style={{ flex: 1 }}>
                                <ThemedText style={{
                                    fontSize: 15,
                                    color: offlineMode ? 'rgba(255,255,255,0.3)' : styles.white,
                                }}>
                                    {t('add-podcast', { defaultValue: 'Podcast hinzufügen' })}
                                </ThemedText>
                                <ThemedText style={{
                                    fontSize: 12,
                                    color: offlineMode ? 'rgba(255,255,255,0.2)' : styles.gray,
                                    marginTop: 2,
                                }}>
                                    {t('add-podcast-description', { defaultValue: 'Suche oder importiere neue Podcasts' })}
                                </ThemedText>
                            </View>
                            {!offlineMode && <Ionicons name="chevron-forward" size={16} color={styles.gray} />}
                            {offlineMode && <Ionicons name="lock-closed" size={16} color="rgba(255,255,255,0.3)" />}
                        </TouchableOpacity>

                        {/* Users Management */}
                        <TouchableOpacity
                            style={{
                                flexDirection: 'row',
                                alignItems: 'center',
                                gap: 12,
                                paddingVertical: 12,
                                borderTopWidth: 1,
                                borderTopColor: 'rgba(255,255,255,0.1)',
                            }}
                            onPress={() => router.push('/users')}
                            disabled={offlineMode}
                        >
                            <Ionicons name="people" size={20} color={offlineMode ? 'rgba(255,255,255,0.3)' : styles.gray} />
                            <View style={{ flex: 1 }}>
                                <ThemedText style={{
                                    fontSize: 15,
                                    color: offlineMode ? 'rgba(255,255,255,0.3)' : styles.white,
                                }}>
                                    {t('manage-users', { defaultValue: 'Benutzer verwalten' })}
                                </ThemedText>
                                <ThemedText style={{
                                    fontSize: 12,
                                    color: offlineMode ? 'rgba(255,255,255,0.2)' : styles.gray,
                                    marginTop: 2,
                                }}>
                                    {t('manage-users-description', { defaultValue: 'Alle Benutzer anzeigen und verwalten' })}
                                </ThemedText>
                            </View>
                            {!offlineMode && <Ionicons name="chevron-forward" size={16} color={styles.gray} />}
                            {offlineMode && <Ionicons name="lock-closed" size={16} color="rgba(255,255,255,0.3)" />}
                        </TouchableOpacity>

                        {/* Invites Management */}
                        <TouchableOpacity
                            style={{
                                flexDirection: 'row',
                                alignItems: 'center',
                                gap: 12,
                                paddingVertical: 12,
                                borderTopWidth: 1,
                                borderTopColor: 'rgba(255,255,255,0.1)',
                            }}
                            onPress={() => router.push('/invites')}
                            disabled={offlineMode}
                        >
                            <Ionicons name="mail" size={20} color={offlineMode ? 'rgba(255,255,255,0.3)' : styles.gray} />
                            <View style={{ flex: 1 }}>
                                <ThemedText style={{
                                    fontSize: 15,
                                    color: offlineMode ? 'rgba(255,255,255,0.3)' : styles.white,
                                }}>
                                    {t('manage-invites', { defaultValue: 'Einladungen verwalten' })}
                                </ThemedText>
                                <ThemedText style={{
                                    fontSize: 12,
                                    color: offlineMode ? 'rgba(255,255,255,0.2)' : styles.gray,
                                    marginTop: 2,
                                }}>
                                    {t('manage-invites-description', { defaultValue: 'Neue Benutzer einladen und Einladungen verwalten' })}
                                </ThemedText>
                            </View>
                            {!offlineMode && <Ionicons name="chevron-forward" size={16} color={styles.gray} />}
                            {offlineMode && <Ionicons name="lock-closed" size={16} color="rgba(255,255,255,0.3)" />}
                        </TouchableOpacity>
                    </View>
                )}

                {/* User Profile Section - nur bei Basic Auth anzeigen */}
                {authType === 'basic' && userProfile && (
                    <View style={{
                        backgroundColor: styles.darkColor,
                        borderRadius: 10,
                        padding: 15,
                        marginTop: 20,
                    }}>
                        <View style={{
                            flexDirection: 'row',
                            alignItems: 'center',
                            gap: 12,
                            marginBottom: 15,
                        }}>
                            <Ionicons name="person-circle" size={40} color={styles.accentColor} />
                            <View style={{ flex: 1 }}>
                                <ThemedText style={{
                                    fontSize: 18,
                                    color: styles.white,
                                    fontWeight: '600'
                                }}>
                                    {userProfile.username}
                                </ThemedText>
                                <ThemedText style={{
                                    fontSize: 13,
                                    color: styles.gray,
                                }}>
                                    {userProfile.role === 'admin'
                                        ? t('role-admin', { defaultValue: 'Administrator' })
                                        : t('role-user', { defaultValue: 'Benutzer' })
                                    }
                                    {userProfile.readOnly && (
                                        <Text style={{ color: styles.gray }}> • {t('read-only', { defaultValue: 'Nur Lesen' })}</Text>
                                    )}
                                </ThemedText>
                            </View>
                            <TouchableOpacity
                                style={{
                                    backgroundColor: 'rgba(255,255,255,0.1)',
                                    borderRadius: 8,
                                    padding: 10,
                                }}
                                onPress={handleRefreshProfile}
                                disabled={isRefreshing}
                            >
                                {isRefreshing ? (
                                    <ActivityIndicator size="small" color={styles.accentColor} />
                                ) : (
                                    <Ionicons name="refresh" size={20} color={styles.accentColor} />
                                )}
                            </TouchableOpacity>
                        </View>

                        {/* Password Change Section - nur wenn nicht readOnly */}
                        {!userProfile.readOnly && (
                            <View style={{
                                borderTopWidth: 1,
                                borderTopColor: 'rgba(255,255,255,0.1)',
                                paddingTop: 15,
                            }}>
                                {!showPasswordChange ? (
                                    <TouchableOpacity
                                        style={{
                                            flexDirection: 'row',
                                            alignItems: 'center',
                                            gap: 10,
                                            paddingVertical: 8,
                                        }}
                                        onPress={() => setShowPasswordChange(true)}
                                    >
                                        <Ionicons name="key-outline" size={20} color={styles.gray} />
                                        <ThemedText style={{
                                            fontSize: 15,
                                            color: styles.white,
                                        }}>
                                            {t('change-password', { defaultValue: 'Passwort ändern' })}
                                        </ThemedText>
                                        <Ionicons name="chevron-forward" size={16} color={styles.gray} style={{ marginLeft: 'auto' }} />
                                    </TouchableOpacity>
                                ) : (
                                    <View>
                                        <ThemedText style={{
                                            fontSize: 14,
                                            color: styles.gray,
                                            marginBottom: 10,
                                        }}>
                                            {t('change-password', { defaultValue: 'Passwort ändern' })}
                                        </ThemedText>
                                        <TextInput
                                            style={{
                                                backgroundColor: 'rgba(255,255,255,0.05)',
                                                borderRadius: 8,
                                                padding: 12,
                                                color: styles.white,
                                                fontSize: 14,
                                                marginBottom: 10,
                                            }}
                                            placeholder={t('new-password', { defaultValue: 'Neues Passwort' })}
                                            placeholderTextColor={styles.gray}
                                            secureTextEntry
                                            value={newPassword}
                                            onChangeText={setNewPassword}
                                        />
                                        <TextInput
                                            style={{
                                                backgroundColor: 'rgba(255,255,255,0.05)',
                                                borderRadius: 8,
                                                padding: 12,
                                                color: styles.white,
                                                fontSize: 14,
                                                marginBottom: 10,
                                            }}
                                            placeholder={t('confirm-password', { defaultValue: 'Passwort bestätigen' })}
                                            placeholderTextColor={styles.gray}
                                            secureTextEntry
                                            value={confirmPassword}
                                            onChangeText={setConfirmPassword}
                                        />
                                        <View style={{
                                            flexDirection: 'row',
                                            gap: 10,
                                        }}>
                                            <TouchableOpacity
                                                style={{
                                                    flex: 1,
                                                    backgroundColor: 'rgba(255,255,255,0.1)',
                                                    borderRadius: 8,
                                                    padding: 12,
                                                    alignItems: 'center',
                                                }}
                                                onPress={() => {
                                                    setShowPasswordChange(false);
                                                    setNewPassword('');
                                                    setConfirmPassword('');
                                                }}
                                            >
                                                <ThemedText style={{ color: styles.white }}>
                                                    {t('cancel')}
                                                </ThemedText>
                                            </TouchableOpacity>
                                            <TouchableOpacity
                                                style={{
                                                    flex: 1,
                                                    backgroundColor: styles.accentColor,
                                                    borderRadius: 8,
                                                    padding: 12,
                                                    alignItems: 'center',
                                                }}
                                                onPress={handleChangePassword}
                                                disabled={isUpdating}
                                            >
                                                {isUpdating ? (
                                                    <ActivityIndicator size="small" color={styles.white} />
                                                ) : (
                                                    <ThemedText style={{ color: styles.white }}>
                                                        {t('save', { defaultValue: 'Speichern' })}
                                                    </ThemedText>
                                                )}
                                            </TouchableOpacity>
                                        </View>
                                    </View>
                                )}
                            </View>
                        )}

                        {/* API Key */}
                        {userApiKey && (
                            <View style={{
                                borderTopWidth: 1,
                                borderTopColor: 'rgba(255,255,255,0.1)',
                                paddingTop: 15,
                                marginTop: 15,
                            }}>
                                <ThemedText style={{
                                    fontSize: 14,
                                    color: styles.gray,
                                    marginBottom: 8
                                }}>
                                    {t('api-key', { defaultValue: 'API-Key' })}
                                </ThemedText>
                                <View style={{
                                    flexDirection: 'row',
                                    alignItems: 'center',
                                    gap: 10,
                                }}>
                                    <View style={{
                                        flex: 1,
                                        backgroundColor: 'rgba(255,255,255,0.05)',
                                        borderRadius: 8,
                                        padding: 12,
                                    }}>
                                        <Text
                                            style={{
                                                color: styles.white,
                                                fontSize: 14,
                                                fontFamily: 'monospace',
                                            }}
                                            numberOfLines={1}
                                            ellipsizeMode="middle"
                                        >
                                            {userApiKey}
                                        </Text>
                                    </View>
                                    <TouchableOpacity
                                        style={{
                                            backgroundColor: styles.accentColor,
                                            borderRadius: 8,
                                            padding: 12,
                                        }}
                                        onPress={handleCopyApiKey}
                                    >
                                        <Ionicons name="copy-outline" size={20} color={styles.white} />
                                    </TouchableOpacity>
                                </View>
                                <ThemedText style={{
                                    fontSize: 12,
                                    color: styles.gray,
                                    marginTop: 8,
                                }}>
                                    {t('api-key-hint', { defaultValue: 'Dieser Key wird für den Zugriff auf geschützte Medien verwendet.' })}
                                </ThemedText>

                                {/* Generate new API Key - nur wenn nicht readOnly */}
                                {!userProfile.readOnly && (
                                    <TouchableOpacity
                                        style={{
                                            flexDirection: 'row',
                                            alignItems: 'center',
                                            justifyContent: 'center',
                                            gap: 8,
                                            marginTop: 12,
                                            paddingVertical: 10,
                                            backgroundColor: 'rgba(255,255,255,0.1)',
                                            borderRadius: 8,
                                        }}
                                        onPress={handleGenerateNewApiKey}
                                        disabled={isUpdating}
                                    >
                                        {isUpdating ? (
                                            <ActivityIndicator size="small" color={styles.accentColor} />
                                        ) : (
                                            <>
                                                <Ionicons name="refresh" size={16} color={styles.accentColor} />
                                                <ThemedText style={{
                                                    fontSize: 14,
                                                    color: styles.accentColor,
                                                }}>
                                                    {t('generate-new-api-key', { defaultValue: 'Neuen API-Key generieren' })}
                                                </ThemedText>
                                            </>
                                        )}
                                    </TouchableOpacity>
                                )}
                            </View>
                        )}
                    </View>
                )}

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
            </ScrollView>
        </SafeAreaView>
    );
};

export default SettingsScreen;
