import { View, TouchableOpacity, Alert, ActivityIndicator, ScrollView, Text } from 'react-native';
import { useTranslation } from 'react-i18next';
import { Ionicons } from '@expo/vector-icons';
import { useState } from 'react';
import { ThemedText } from '@/components/ThemedText';
import { styles } from '@/styles/styles';
import { $api } from '@/client';
import { useStore } from '@/store/store';

const UsersScreen = () => {
    const { t } = useTranslation();
    const serverUrl = useStore((state) => state.serverUrl);
    const basicAuthUsername = useStore((state) => state.basicAuthUsername);
    const [isDeleting, setIsDeleting] = useState<number | null>(null);

    const { data: users, isLoading, refetch } = $api.useQuery('get', '/api/v1/users', {}, {
        enabled: !!serverUrl,
    });

    const deleteUserMutation = $api.useMutation('delete', '/api/v1/users/{username}', {
        onSuccess: () => {
            Alert.alert(
                t('success', { defaultValue: 'Erfolg' }),
                t('user-deleted', { defaultValue: 'Benutzer wurde gelöscht.' })
            );
            refetch();
            setIsDeleting(null);
        },
        onError: (error) => {
            console.error('Failed to delete user:', error);
            Alert.alert(
                t('error', { defaultValue: 'Fehler' }),
                t('delete-user-error', { defaultValue: 'Benutzer konnte nicht gelöscht werden.' })
            );
            setIsDeleting(null);
        }
    });

    const handleDeleteUser = (username: string, userId: number) => {
        Alert.alert(
            t('delete-user-confirm-title', { defaultValue: 'Benutzer löschen?' }),
            t('delete-user-confirm-message', { defaultValue: `Möchtest du den Benutzer "${username}" wirklich löschen?` }),
            [
                {
                    text: t('cancel'),
                    style: 'cancel',
                },
                {
                    text: t('delete', { defaultValue: 'Löschen' }),
                    style: 'destructive',
                    onPress: () => {
                        setIsDeleting(userId);
                        deleteUserMutation.mutate({
                            params: {
                                path: { username },
                            },
                        });
                    },
                },
            ]
        );
    };

    const getRoleBadgeColor = (role: string) => {
        return role === 'admin' ? styles.accentColor : styles.gray;
    };

    return (
        <ScrollView
            style={{ flex: 1, backgroundColor: styles.lightDarkColor }}
            contentContainerStyle={{ paddingTop: 20, paddingHorizontal: 10, paddingBottom: 40 }}
        >

                {isLoading ? (
                    <View style={{ padding: 40, alignItems: 'center' }}>
                        <ActivityIndicator size="large" color={styles.accentColor} />
                    </View>
                ) : !users || users.length === 0 ? (
                    <View style={{
                        backgroundColor: styles.darkColor,
                        borderRadius: 10,
                        padding: 20,
                        alignItems: 'center',
                    }}>
                        <Ionicons name="people-outline" size={48} color={styles.gray} />
                        <ThemedText style={{ color: styles.gray, marginTop: 10 }}>
                            {t('no-users', { defaultValue: 'Keine Benutzer vorhanden.' })}
                        </ThemedText>
                    </View>
                ) : (
                    <View style={{ gap: 10 }}>
                        {users.map((user) => {
                            const isCurrentUser = user.username === basicAuthUsername;
                            const isReadOnly = user.role === 'admin'; // ReadOnly users have 'uploader' role
                            const canDelete = !isCurrentUser && !isReadOnly;

                            return (
                            <View
                                key={user.id}
                                style={{
                                    backgroundColor: styles.darkColor,
                                    borderRadius: 10,
                                    padding: 15,
                                    flexDirection: 'row',
                                    alignItems: 'center',
                                }}
                            >
                                <Ionicons name="person-circle" size={40} color={styles.accentColor} />
                                <View style={{ flex: 1, marginLeft: 12 }}>
                                    <View style={{ flexDirection: 'row', alignItems: 'center', gap: 8 }}>
                                        <ThemedText style={{
                                            fontSize: 16,
                                            color: styles.white,
                                            fontWeight: '600',
                                        }}>
                                            {user.username}
                                        </ThemedText>
                                        {isCurrentUser && (
                                            <View style={{
                                                backgroundColor: 'rgba(33,150,243,0.2)',
                                                paddingHorizontal: 6,
                                                paddingVertical: 2,
                                                borderRadius: 4,
                                            }}>
                                                <Text style={{
                                                    fontSize: 10,
                                                    color: '#2196F3',
                                                    fontWeight: '600',
                                                }}>
                                                    {t('you', { defaultValue: 'Du' })}
                                                </Text>
                                            </View>
                                        )}
                                    </View>
                                    <View style={{ flexDirection: 'row', alignItems: 'center', marginTop: 4, gap: 8 }}>
                                        <View style={{
                                            backgroundColor: getRoleBadgeColor(user.role),
                                            paddingHorizontal: 8,
                                            paddingVertical: 2,
                                            borderRadius: 4,
                                        }}>
                                            <Text style={{
                                                fontSize: 12,
                                                color: styles.white,
                                                fontWeight: '600',
                                            }}>
                                                {user.role === 'admin'
                                                    ? t('role-admin', { defaultValue: 'Admin' })
                                                    : user.role === 'uploader'
                                                    ? t('role-readonly', { defaultValue: 'Nur Lesen' })
                                                    : t('role-user', { defaultValue: 'Benutzer' })
                                                }
                                            </Text>
                                        </View>
                                        <ThemedText style={{ fontSize: 12, color: styles.gray }}>
                                            {t('created-at', { defaultValue: 'Erstellt' })}: {new Date(user.createdAt).toLocaleDateString()}
                                        </ThemedText>
                                    </View>
                                    {isReadOnly && (
                                        <ThemedText style={{ fontSize: 11, color: styles.gray, marginTop: 4, fontStyle: 'italic' }}>
                                            {t('managed-by-config', { defaultValue: 'Wird durch Konfiguration verwaltet' })}
                                        </ThemedText>
                                    )}
                                </View>
                                {canDelete ? (
                                    <TouchableOpacity
                                        onPress={() => handleDeleteUser(user.username, user.id)}
                                        disabled={isDeleting === user.id}
                                        style={{
                                            backgroundColor: 'rgba(255,68,68,0.2)',
                                            borderRadius: 8,
                                            padding: 10,
                                        }}
                                    >
                                        {isDeleting === user.id ? (
                                            <ActivityIndicator size="small" color="#ff4444" />
                                        ) : (
                                            <Ionicons name="trash-outline" size={20} color="#ff4444" />
                                        )}
                                    </TouchableOpacity>
                                ) : (
                                    <View
                                        style={{
                                            backgroundColor: 'rgba(255,255,255,0.05)',
                                            borderRadius: 8,
                                            padding: 10,
                                            opacity: 0.5,
                                        }}
                                    >
                                        <Ionicons name="lock-closed" size={20} color={styles.gray} />
                                    </View>
                                )}
                            </View>
                        )})}
                    </View>
                )}
            </ScrollView>
    );
};

export default UsersScreen;

