import { View, TouchableOpacity, Alert, ActivityIndicator, ScrollView, Text } from 'react-native';
import { useTranslation } from 'react-i18next';
import { Ionicons } from '@expo/vector-icons';
import { useState } from 'react';
import * as Clipboard from 'expo-clipboard';
import { ThemedText } from '@/components/ThemedText';
import { styles } from '@/styles/styles';
import { $api } from '@/client';
import { useStore } from '@/store/store';

const InvitesScreen = () => {
    const { t } = useTranslation();
    const serverUrl = useStore((state) => state.serverUrl);
    const [isDeleting, setIsDeleting] = useState<string | null>(null);
    const [showCreateInvite, setShowCreateInvite] = useState(false);
    const [newInviteRole, setNewInviteRole] = useState<'admin' | 'user'>('user');
    const [isCreating, setIsCreating] = useState(false);

    const { data: invites, isLoading, refetch } = $api.useQuery('get', '/api/v1/invites', {}, {
        enabled: !!serverUrl,
    });

    const createInviteMutation = $api.useMutation('post', '/api/v1/invites', {
        onSuccess: (data) => {
            if (data) {
                const inviteLink = `${serverUrl}/ui/invite/${data.id}`;
                Clipboard.setStringAsync(inviteLink);
                Alert.alert(
                    t('success', { defaultValue: 'Erfolg' }),
                    t('invite-created-copied', { defaultValue: 'Einladung wurde erstellt und Link in die Zwischenablage kopiert.' })
                );
                setShowCreateInvite(false);
                setNewInviteRole('user');
                refetch();
            }
            setIsCreating(false);
        },
        onError: (error) => {
            console.error('Failed to create invite:', error);
            Alert.alert(
                t('error', { defaultValue: 'Fehler' }),
                t('create-invite-error', { defaultValue: 'Einladung konnte nicht erstellt werden.' })
            );
            setIsCreating(false);
        }
    });

    const handleCreateInvite = () => {
        setIsCreating(true);
        const inviteData = {
            role: newInviteRole,
            explicitConsent: false,
        };
        console.log('Creating invite with data:', inviteData);
        createInviteMutation.mutate({
            body: inviteData
        } as any);
    };

    const deleteInviteMutation = $api.useMutation('delete', '/api/v1/invites/{invite_id}', {
        onSuccess: () => {
            Alert.alert(
                t('success', { defaultValue: 'Erfolg' }),
                t('invite-deleted', { defaultValue: 'Einladung wurde gelöscht.' })
            );
            refetch();
            setIsDeleting(null);
        },
        onError: (error) => {
            console.error('Failed to delete invite:', error);
            Alert.alert(
                t('error', { defaultValue: 'Fehler' }),
                t('delete-invite-error', { defaultValue: 'Einladung konnte nicht gelöscht werden.' })
            );
            setIsDeleting(null);
        }
    });

    const handleDeleteInvite = (inviteId: string) => {
        Alert.alert(
            t('delete-invite-confirm-title', { defaultValue: 'Einladung löschen?' }),
            t('delete-invite-confirm-message', { defaultValue: 'Möchtest du diese Einladung wirklich löschen?' }),
            [
                {
                    text: t('cancel'),
                    style: 'cancel',
                },
                {
                    text: t('delete', { defaultValue: 'Löschen' }),
                    style: 'destructive',
                    onPress: () => {
                        setIsDeleting(inviteId);
                        deleteInviteMutation.mutate({
                            params: {
                                path: { invite_id: inviteId },
                            },
                        });
                    },
                },
            ]
        );
    };

    const handleCopyInviteLink = async (inviteId: string) => {
        const inviteLink = `${serverUrl}/ui/invite/${inviteId}`;
        await Clipboard.setStringAsync(inviteLink);
        Alert.alert(
            t('copied', { defaultValue: 'Kopiert' }),
            t('invite-link-copied', { defaultValue: 'Einladungslink wurde in die Zwischenablage kopiert.' })
        );
    };

    const isInviteExpired = (expiresAt: string) => {
        return new Date(expiresAt) < new Date();
    };

    const isInviteAccepted = (acceptedAt: string | null | undefined) => {
        return !!acceptedAt;
    };

    return (
        <ScrollView
            style={{ flex: 1, backgroundColor: styles.lightDarkColor }}
            contentContainerStyle={{ paddingTop: 20, paddingHorizontal: 10, paddingBottom: 40 }}
        >

                {!showCreateInvite && (
                    <TouchableOpacity
                        onPress={() => setShowCreateInvite(true)}
                        style={{
                            backgroundColor: styles.accentColor,
                            borderRadius: 10,
                            padding: 15,
                            marginBottom: 20,
                            flexDirection: 'row',
                            alignItems: 'center',
                            justifyContent: 'center',
                            gap: 10,
                        }}
                    >
                        <Ionicons name="add-circle-outline" size={24} color={styles.white} />
                        <ThemedText style={{
                            color: styles.white,
                            fontSize: 16,
                            fontWeight: '600',
                        }}>
                            {t('create-invite', { defaultValue: 'Neue Einladung erstellen' })}
                        </ThemedText>
                    </TouchableOpacity>
                )}

                {/* Create Invite Form */}
                {showCreateInvite && (
                    <View style={{
                        backgroundColor: styles.darkColor,
                        borderRadius: 10,
                        padding: 15,
                        marginBottom: 20,
                    }}>
                        <ThemedText style={{
                            fontSize: 16,
                            color: styles.white,
                            fontWeight: '600',
                            marginBottom: 15,
                        }}>
                            {t('create-invite', { defaultValue: 'Neue Einladung erstellen' })}
                        </ThemedText>

                        <ThemedText style={{
                            fontSize: 14,
                            color: styles.gray,
                            marginBottom: 8,
                        }}>
                            {t('role', { defaultValue: 'Rolle' })}
                        </ThemedText>

                        <View style={{ flexDirection: 'row', gap: 10, marginBottom: 15 }}>
                            <TouchableOpacity
                                onPress={() => setNewInviteRole('user')}
                                style={{
                                    flex: 1,
                                    backgroundColor: newInviteRole === 'user' ? styles.accentColor : 'rgba(255,255,255,0.1)',
                                    borderRadius: 8,
                                    padding: 12,
                                    alignItems: 'center',
                                }}
                            >
                                <ThemedText style={{ color: styles.white }}>
                                    {t('role-user', { defaultValue: 'Benutzer' })}
                                </ThemedText>
                            </TouchableOpacity>
                            <TouchableOpacity
                                onPress={() => setNewInviteRole('admin')}
                                style={{
                                    flex: 1,
                                    backgroundColor: newInviteRole === 'admin' ? styles.accentColor : 'rgba(255,255,255,0.1)',
                                    borderRadius: 8,
                                    padding: 12,
                                    alignItems: 'center',
                                }}
                            >
                                <ThemedText style={{ color: styles.white }}>
                                    {t('role-admin', { defaultValue: 'Admin' })}
                                </ThemedText>
                            </TouchableOpacity>
                        </View>

                        <View style={{ flexDirection: 'row', gap: 10 }}>
                            <TouchableOpacity
                                onPress={() => {
                                    setShowCreateInvite(false);
                                    setNewInviteRole('user');
                                }}
                                style={{
                                    flex: 1,
                                    backgroundColor: 'rgba(255,255,255,0.1)',
                                    borderRadius: 8,
                                    padding: 12,
                                    alignItems: 'center',
                                }}
                            >
                                <ThemedText style={{ color: styles.white }}>
                                    {t('cancel')}
                                </ThemedText>
                            </TouchableOpacity>
                            <TouchableOpacity
                                onPress={handleCreateInvite}
                                disabled={isCreating}
                                style={{
                                    flex: 1,
                                    backgroundColor: styles.accentColor,
                                    borderRadius: 8,
                                    padding: 12,
                                    alignItems: 'center',
                                }}
                            >
                                {isCreating ? (
                                    <ActivityIndicator size="small" color={styles.white} />
                                ) : (
                                    <ThemedText style={{ color: styles.white }}>
                                        {t('create', { defaultValue: 'Erstellen' })}
                                    </ThemedText>
                                )}
                            </TouchableOpacity>
                        </View>
                    </View>
                )}

                {/* Invites List */}
                {isLoading ? (
                    <View style={{ padding: 40, alignItems: 'center' }}>
                        <ActivityIndicator size="large" color={styles.accentColor} />
                    </View>
                ) : !invites || invites.length === 0 ? (
                    <View style={{
                        backgroundColor: styles.darkColor,
                        borderRadius: 10,
                        padding: 20,
                        alignItems: 'center',
                    }}>
                        <Ionicons name="mail-outline" size={48} color={styles.gray} />
                        <ThemedText style={{ color: styles.gray, marginTop: 10 }}>
                            {t('no-invites', { defaultValue: 'Keine Einladungen vorhanden.' })}
                        </ThemedText>
                    </View>
                ) : (
                    <View style={{ gap: 10 }}>
                        {invites.map((invite) => {
                            const expired = isInviteExpired(invite.expiresAt);
                            const accepted = isInviteAccepted(invite.acceptedAt);

                            return (
                                <View
                                    key={invite.id}
                                    style={{
                                        backgroundColor: styles.darkColor,
                                        borderRadius: 10,
                                        padding: 15,
                                        opacity: expired || accepted ? 0.6 : 1,
                                    }}
                                >
                                    <View style={{ flexDirection: 'row', alignItems: 'center', marginBottom: 10 }}>
                                        <Ionicons
                                            name={accepted ? "checkmark-circle" : expired ? "time-outline" : "mail"}
                                            size={32}
                                            color={accepted ? '#4CAF50' : expired ? styles.gray : styles.accentColor}
                                        />
                                        <View style={{ flex: 1, marginLeft: 12 }}>
                                            <View style={{ flexDirection: 'row', alignItems: 'center', gap: 8 }}>
                                                <View style={{
                                                    backgroundColor: invite.role === 'admin' ? styles.accentColor : styles.gray,
                                                    paddingHorizontal: 8,
                                                    paddingVertical: 2,
                                                    borderRadius: 4,
                                                }}>
                                                    <Text style={{
                                                        fontSize: 12,
                                                        color: styles.white,
                                                        fontWeight: '600',
                                                    }}>
                                                        {invite.role === 'admin'
                                                            ? t('role-admin', { defaultValue: 'Admin' })
                                                            : t('role-user', { defaultValue: 'Benutzer' })
                                                        }
                                                    </Text>
                                                </View>
                                                {accepted && (
                                                    <View style={{
                                                        backgroundColor: '#4CAF50',
                                                        paddingHorizontal: 8,
                                                        paddingVertical: 2,
                                                        borderRadius: 4,
                                                    }}>
                                                        <Text style={{
                                                            fontSize: 12,
                                                            color: styles.white,
                                                            fontWeight: '600',
                                                        }}>
                                                            {t('accepted', { defaultValue: 'Akzeptiert' })}
                                                        </Text>
                                                    </View>
                                                )}
                                                {expired && !accepted && (
                                                    <View style={{
                                                        backgroundColor: '#ff4444',
                                                        paddingHorizontal: 8,
                                                        paddingVertical: 2,
                                                        borderRadius: 4,
                                                    }}>
                                                        <Text style={{
                                                            fontSize: 12,
                                                            color: styles.white,
                                                            fontWeight: '600',
                                                        }}>
                                                            {t('expired', { defaultValue: 'Abgelaufen' })}
                                                        </Text>
                                                    </View>
                                                )}
                                            </View>
                                            <ThemedText style={{ fontSize: 12, color: styles.gray, marginTop: 4 }}>
                                                {t('created-at', { defaultValue: 'Erstellt' })}: {new Date(invite.createdAt).toLocaleDateString()}
                                            </ThemedText>
                                            {accepted && invite.acceptedAt && (
                                                <ThemedText style={{ fontSize: 12, color: styles.gray }}>
                                                    {t('accepted-at', { defaultValue: 'Akzeptiert' })}: {new Date(invite.acceptedAt).toLocaleDateString()}
                                                </ThemedText>
                                            )}
                                            {!accepted && (
                                                <ThemedText style={{ fontSize: 12, color: styles.gray }}>
                                                    {t('expires-at', { defaultValue: 'Läuft ab' })}: {new Date(invite.expiresAt).toLocaleDateString()}
                                                </ThemedText>
                                            )}
                                        </View>
                                    </View>

                                    <View style={{ flexDirection: 'row', gap: 10, marginTop: 10 }}>
                                        {!expired && !accepted && (
                                            <TouchableOpacity
                                                onPress={() => handleCopyInviteLink(invite.id)}
                                                style={{
                                                    flex: 1,
                                                    backgroundColor: styles.accentColor,
                                                    borderRadius: 8,
                                                    padding: 10,
                                                    flexDirection: 'row',
                                                    alignItems: 'center',
                                                    justifyContent: 'center',
                                                    gap: 8,
                                                }}
                                            >
                                                <Ionicons name="copy-outline" size={16} color={styles.white} />
                                                <ThemedText style={{ color: styles.white, fontSize: 14 }}>
                                                    {t('copy-link', { defaultValue: 'Link kopieren' })}
                                                </ThemedText>
                                            </TouchableOpacity>
                                        )}
                                        {accepted ? (
                                            <View
                                                style={{
                                                    flex: 1,
                                                    backgroundColor: 'rgba(255,255,255,0.05)',
                                                    borderRadius: 8,
                                                    padding: 10,
                                                    flexDirection: 'row',
                                                    alignItems: 'center',
                                                    justifyContent: 'center',
                                                    gap: 8,
                                                    opacity: 0.5,
                                                }}
                                            >
                                                <Ionicons name="lock-closed" size={16} color={styles.gray} />
                                                <ThemedText style={{ color: styles.gray, fontSize: 14 }}>
                                                    {t('cannot-delete-accepted', { defaultValue: 'Kann nicht gelöscht werden' })}
                                                </ThemedText>
                                            </View>
                                        ) : (
                                            <TouchableOpacity
                                                onPress={() => handleDeleteInvite(invite.id)}
                                                disabled={isDeleting === invite.id}
                                                style={{
                                                    backgroundColor: 'rgba(255,68,68,0.2)',
                                                    borderRadius: 8,
                                                    padding: 10,
                                                    paddingHorizontal: !expired && !accepted ? 15 : undefined,
                                                    flex: expired ? 1 : undefined,
                                                    alignItems: 'center',
                                                    justifyContent: 'center',
                                                }}
                                            >
                                                {isDeleting === invite.id ? (
                                                    <ActivityIndicator size="small" color="#ff4444" />
                                                ) : (
                                                    <View style={{ flexDirection: 'row', alignItems: 'center', gap: 8 }}>
                                                        <Ionicons name="trash-outline" size={16} color="#ff4444" />
                                                        {expired && (
                                                            <ThemedText style={{ color: '#ff4444', fontSize: 14 }}>
                                                                {t('delete', { defaultValue: 'Löschen' })}
                                                            </ThemedText>
                                                        )}
                                                    </View>
                                                )}
                                            </TouchableOpacity>
                                        )}
                                    </View>
                                </View>
                            );
                        })}
                    </View>
                )}
            </ScrollView>
    );
};

export default InvitesScreen;

