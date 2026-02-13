import { View, TouchableOpacity, TextInput, ScrollView, ActivityIndicator, Alert, Image, KeyboardAvoidingView, Platform } from 'react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
import { useTranslation } from 'react-i18next';
import { useRouter } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import { useState, useCallback } from 'react';
import * as DocumentPicker from 'expo-document-picker';
import * as FileSystem from 'expo-file-system';
import { ThemedText } from '@/components/ThemedText';
import { styles } from '@/styles/styles';
import { useStore } from '@/store/store';
import { $api } from '@/client';
import { components } from '@/schema';

type SearchSource = 'itunes' | 'podindex' | 'rss' | 'opml';

type ItunesResult = components["schemas"]["ItunesModel"];
type PodindexResult = components["schemas"]["Feed"];

const AddPodcastScreen = () => {
    const { t } = useTranslation();
    const router = useRouter();
    const serverConfig = useStore((state) => state.serverConfig);
    const userProfile = useStore((state) => state.userProfile);

    const [activeTab, setActiveTab] = useState<SearchSource>('itunes');
    const [searchQuery, setSearchQuery] = useState('');
    const [rssUrl, setRssUrl] = useState('');
    const [isAdding, setIsAdding] = useState(false);
    const [searchResults, setSearchResults] = useState<ItunesResult[] | PodindexResult[]>([]);
    const [addingPodcastId, setAddingPodcastId] = useState<number | null>(null);

    const podindexAvailable = serverConfig?.podindexConfigured ?? false;

    // Search mutation for iTunes (type_of = 0) and Podindex (type_of = 1)
    const searchMutation = $api.useMutation('get', '/api/v1/podcasts/{type_of}/{podcast}/search');

    // Add podcast mutations
    const addItunesMutation = $api.useMutation('post', '/api/v1/podcasts/itunes');
    const addPodindexMutation = $api.useMutation('post', '/api/v1/podcasts/podindex');
    const addRssMutation = $api.useMutation('post', '/api/v1/podcasts/feed');
    const importOpmlMutation = $api.useMutation('post', '/api/v1/podcasts/opml');

    // Search podcasts via iTunes or Podindex
    const handleSearch = useCallback(async () => {
        if (!searchQuery.trim()) return;

        setSearchResults([]);

        try {
            const typeOf = activeTab === 'itunes' ? 0 : 1;
            const data = await searchMutation.mutateAsync({
                params: {
                    path: {
                        type_of: typeOf,
                        podcast: searchQuery.trim(),
                    },
                },
            });

            if (activeTab === 'itunes' && 'results' in data) {
                setSearchResults(data.results as ItunesResult[]);
            } else if (activeTab === 'podindex' && 'feeds' in data) {
                setSearchResults(data.feeds as PodindexResult[]);
            }
        } catch (error) {
            console.error('Search error:', error);
            Alert.alert(
                t('error', { defaultValue: 'Fehler' }),
                t('search-failed', { defaultValue: 'Suche fehlgeschlagen. Bitte versuche es erneut.' })
            );
        }
    }, [searchQuery, activeTab, t, searchMutation]);

    // Add podcast from iTunes
    const handleAddFromItunes = useCallback(async (podcast: ItunesResult) => {
        if (!podcast.collectionId) return;

        setAddingPodcastId(podcast.collectionId);
        setIsAdding(true);

        try {
            await addItunesMutation.mutateAsync({
                body: {
                    trackId: podcast.collectionId,
                    userId: userProfile?.id ?? 0,
                },
            });

            Alert.alert(
                t('success', { defaultValue: 'Erfolg' }),
                t('podcast-added', { defaultValue: 'Podcast wurde hinzugefügt!' }),
                [
                    {
                        text: 'OK',
                        onPress: () => router.back(),
                    },
                ]
            );
        } catch (error) {
            console.error('Add podcast error:', error);
            Alert.alert(
                t('error', { defaultValue: 'Fehler' }),
                t('add-podcast-failed', { defaultValue: 'Podcast konnte nicht hinzugefügt werden.' })
            );
        } finally {
            setIsAdding(false);
            setAddingPodcastId(null);
        }
    }, [userProfile, t, router, addItunesMutation]);

    // Add podcast from Podindex
    const handleAddFromPodindex = useCallback(async (podcast: PodindexResult) => {
        if (!podcast.id) return;

        setAddingPodcastId(podcast.id);
        setIsAdding(true);

        try {
            await addPodindexMutation.mutateAsync({
                body: {
                    trackId: podcast.id,
                    userId: userProfile?.id ?? 0,
                },
            });

            Alert.alert(
                t('success', { defaultValue: 'Erfolg' }),
                t('podcast-added', { defaultValue: 'Podcast wurde hinzugefügt!' }),
                [
                    {
                        text: 'OK',
                        onPress: () => router.back(),
                    },
                ]
            );
        } catch (error) {
            console.error('Add podcast error:', error);
            Alert.alert(
                t('error', { defaultValue: 'Fehler' }),
                t('add-podcast-failed', { defaultValue: 'Podcast konnte nicht hinzugefügt werden.' })
            );
        } finally {
            setIsAdding(false);
            setAddingPodcastId(null);
        }
    }, [userProfile, t, router, addPodindexMutation]);

    // Add podcast from RSS URL
    const handleAddFromRss = useCallback(async () => {
        if (!rssUrl.trim()) {
            Alert.alert(
                t('error', { defaultValue: 'Fehler' }),
                t('rss-url-required', { defaultValue: 'Bitte gib eine RSS-Feed-URL ein.' })
            );
            return;
        }

        setIsAdding(true);

        try {
            await addRssMutation.mutateAsync({
                body: {
                    rssFeedUrl: rssUrl.trim(),
                },
            });

            Alert.alert(
                t('success', { defaultValue: 'Erfolg' }),
                t('podcast-added', { defaultValue: 'Podcast wurde hinzugefügt!' }),
                [
                    {
                        text: 'OK',
                        onPress: () => router.back(),
                    },
                ]
            );
        } catch (error) {
            console.error('Add RSS podcast error:', error);
            Alert.alert(
                t('error', { defaultValue: 'Fehler' }),
                t('add-podcast-failed', { defaultValue: 'Podcast konnte nicht hinzugefügt werden.' })
            );
        } finally {
            setIsAdding(false);
        }
    }, [rssUrl, t, router, addRssMutation]);

    // Import OPML file
    const handleImportOpml = useCallback(async () => {
        try {
            const result = await DocumentPicker.getDocumentAsync({
                type: ['text/xml', 'application/xml', 'text/x-opml', '*/*'],
                copyToCacheDirectory: true,
            });

            if (result.canceled || !result.assets?.[0]) {
                return;
            }

            const file = result.assets[0];
            setIsAdding(true);

            // Read file content
            const content = await FileSystem.readAsStringAsync(file.uri);

            await importOpmlMutation.mutateAsync({
                body: {
                    content: content,
                },
            });

            Alert.alert(
                t('success', { defaultValue: 'Erfolg' }),
                t('opml-imported', { defaultValue: 'OPML wurde importiert! Die Podcasts werden jetzt hinzugefügt.' }),
                [
                    {
                        text: 'OK',
                        onPress: () => router.back(),
                    },
                ]
            );
        } catch (error) {
            console.error('OPML import error:', error);
            Alert.alert(
                t('error', { defaultValue: 'Fehler' }),
                t('opml-import-failed', { defaultValue: 'OPML konnte nicht importiert werden.' })
            );
        } finally {
            setIsAdding(false);
        }
    }, [t, router, importOpmlMutation]);

    // Render iTunes search result
    const renderItunesResult = (item: ItunesResult) => (
        <View
            key={item.collectionId}
            style={{
                flexDirection: 'row',
                backgroundColor: styles.darkColor,
                borderRadius: 12,
                padding: 12,
                marginBottom: 12,
                gap: 12,
            }}
        >
            <Image
                source={{ uri: item.artworkUrl100 || item.artworkUrl60 || '' }}
                style={{
                    width: 80,
                    height: 80,
                    borderRadius: 8,
                    backgroundColor: 'rgba(255,255,255,0.1)',
                }}
            />
            <View style={{ flex: 1, justifyContent: 'center' }}>
                <ThemedText
                    style={{
                        fontSize: 16,
                        fontWeight: '600',
                        color: styles.white,
                    }}
                    numberOfLines={2}
                >
                    {item.collectionName || item.trackName}
                </ThemedText>
                <ThemedText
                    style={{
                        fontSize: 13,
                        color: styles.gray,
                        marginTop: 4,
                    }}
                    numberOfLines={1}
                >
                    {item.artistName}
                </ThemedText>
                {item.primaryGenreName && (
                    <ThemedText
                        style={{
                            fontSize: 12,
                            color: styles.accentColor,
                            marginTop: 4,
                        }}
                    >
                        {item.primaryGenreName}
                    </ThemedText>
                )}
            </View>
            <TouchableOpacity
                style={{
                    backgroundColor: styles.accentColor,
                    borderRadius: 8,
                    paddingHorizontal: 16,
                    paddingVertical: 10,
                    alignSelf: 'center',
                }}
                onPress={() => handleAddFromItunes(item)}
                disabled={isAdding}
            >
                {isAdding && addingPodcastId === item.collectionId ? (
                    <ActivityIndicator size="small" color={styles.white} />
                ) : (
                    <Ionicons name="add" size={24} color={styles.white} />
                )}
            </TouchableOpacity>
        </View>
    );

    // Render Podindex search result
    const renderPodindexResult = (item: PodindexResult) => (
        <View
            key={item.id}
            style={{
                flexDirection: 'row',
                backgroundColor: styles.darkColor,
                borderRadius: 12,
                padding: 12,
                marginBottom: 12,
                gap: 12,
            }}
        >
            <Image
                source={{ uri: item.artwork || item.image || '' }}
                style={{
                    width: 80,
                    height: 80,
                    borderRadius: 8,
                    backgroundColor: 'rgba(255,255,255,0.1)',
                }}
            />
            <View style={{ flex: 1, justifyContent: 'center' }}>
                <ThemedText
                    style={{
                        fontSize: 16,
                        fontWeight: '600',
                        color: styles.white,
                    }}
                    numberOfLines={2}
                >
                    {item.title}
                </ThemedText>
                <ThemedText
                    style={{
                        fontSize: 13,
                        color: styles.gray,
                        marginTop: 4,
                    }}
                    numberOfLines={1}
                >
                    {item.author || item.ownerName}
                </ThemedText>
            </View>
            <TouchableOpacity
                style={{
                    backgroundColor: styles.accentColor,
                    borderRadius: 8,
                    paddingHorizontal: 16,
                    paddingVertical: 10,
                    alignSelf: 'center',
                }}
                onPress={() => handleAddFromPodindex(item)}
                disabled={isAdding}
            >
                {isAdding && addingPodcastId === item.id ? (
                    <ActivityIndicator size="small" color={styles.white} />
                ) : (
                    <Ionicons name="add" size={24} color={styles.white} />
                )}
            </TouchableOpacity>
        </View>
    );

    // Tab button component
    const TabButton = ({
        tab,
        icon,
        label,
        disabled = false,
    }: {
        tab: SearchSource;
        icon: keyof typeof Ionicons.glyphMap;
        label: string;
        disabled?: boolean;
    }) => (
        <TouchableOpacity
            style={{
                flex: 1,
                flexDirection: 'column',
                alignItems: 'center',
                paddingVertical: 12,
                backgroundColor: activeTab === tab ? styles.accentColor : 'transparent',
                borderRadius: 8,
                opacity: disabled ? 0.4 : 1,
            }}
            onPress={() => {
                if (!disabled) {
                    setActiveTab(tab);
                    setSearchResults([]);
                    setSearchQuery('');
                    setRssUrl('');
                }
            }}
            disabled={disabled}
        >
            <Ionicons
                name={icon}
                size={20}
                color={activeTab === tab ? styles.white : styles.gray}
            />
            <ThemedText
                style={{
                    fontSize: 11,
                    color: activeTab === tab ? styles.white : styles.gray,
                    marginTop: 4,
                    textAlign: 'center',
                }}
                numberOfLines={1}
            >
                {label}
            </ThemedText>
        </TouchableOpacity>
    );

    return (
        <SafeAreaView style={{ flex: 1, backgroundColor: styles.lightDarkColor }}>
            <KeyboardAvoidingView
                style={{ flex: 1 }}
                behavior={Platform.OS === 'ios' ? 'padding' : undefined}
            >

                {/* Tab Bar */}
                <View
                    style={{
                        flexDirection: 'row',
                        marginHorizontal: 16,
                        backgroundColor: styles.darkColor,
                        borderRadius: 12,
                        padding: 4,
                        gap: 4,
                    }}
                >
                    <TabButton tab="itunes" icon="logo-apple" label="iTunes" />
                    <TabButton
                        tab="podindex"
                        icon="search"
                        label="Podindex"
                        disabled={!podindexAvailable}
                    />
                    <TabButton tab="opml" icon="document-text" label="OPML" />
                    <TabButton tab="rss" icon="link" label="RSS" />
                </View>

                {/* Podindex disabled hint */}
                {activeTab === 'podindex' && !podindexAvailable && (
                    <View
                        style={{
                            marginHorizontal: 16,
                            marginTop: 12,
                            backgroundColor: 'rgba(255,152,0,0.1)',
                            borderLeftWidth: 3,
                            borderLeftColor: '#ff9800',
                            padding: 12,
                            borderRadius: 8,
                            flexDirection: 'row',
                            alignItems: 'center',
                            gap: 10,
                        }}
                    >
                        <Ionicons name="warning" size={20} color="#ff9800" />
                        <ThemedText
                            style={{
                                fontSize: 13,
                                color: '#ff9800',
                                flex: 1,
                            }}
                        >
                            {t('podindex-not-configured', {
                                defaultValue:
                                    'Podindex ist auf diesem Server nicht konfiguriert. Bitte verwende iTunes oder füge einen RSS-Feed hinzu.',
                            })}
                        </ThemedText>
                    </View>
                )}

                <ScrollView
                    style={{ flex: 1 }}
                    contentContainerStyle={{
                        paddingHorizontal: 16,
                        paddingTop: 16,
                        paddingBottom: 100,
                    }}
                    keyboardShouldPersistTaps="handled"
                >
                    {/* iTunes / Podindex Search */}
                    {(activeTab === 'itunes' || activeTab === 'podindex') && (
                        <>
                            <View
                                style={{
                                    flexDirection: 'row',
                                    backgroundColor: styles.darkColor,
                                    borderRadius: 12,
                                    padding: 4,
                                    alignItems: 'center',
                                    marginBottom: 16,
                                }}
                            >
                                <TextInput
                                    style={{
                                        flex: 1,
                                        paddingHorizontal: 16,
                                        paddingVertical: 12,
                                        color: styles.white,
                                        fontSize: 16,
                                    }}
                                    placeholder={t('search-podcast-placeholder', {
                                        defaultValue: 'Podcast suchen...',
                                    })}
                                    placeholderTextColor={styles.gray}
                                    value={searchQuery}
                                    onChangeText={setSearchQuery}
                                    onSubmitEditing={handleSearch}
                                    returnKeyType="search"
                                    autoCorrect={false}
                                />
                                <TouchableOpacity
                                    style={{
                                        backgroundColor: styles.accentColor,
                                        borderRadius: 8,
                                        padding: 12,
                                        marginRight: 4,
                                    }}
                                    onPress={handleSearch}
                                    disabled={searchMutation.isPending || !searchQuery.trim()}
                                >
                                    {searchMutation.isPending ? (
                                        <ActivityIndicator size="small" color={styles.white} />
                                    ) : (
                                        <Ionicons name="search" size={20} color={styles.white} />
                                    )}
                                </TouchableOpacity>
                            </View>

                            {/* Search Results */}
                            {searchResults.length > 0 && (
                                <>
                                    <ThemedText
                                        style={{
                                            fontSize: 14,
                                            color: styles.gray,
                                            marginBottom: 12,
                                        }}
                                    >
                                        {t('search-results', {
                                            defaultValue: '{{count}} Ergebnisse',
                                            count: searchResults.length,
                                        })}
                                    </ThemedText>
                                    {activeTab === 'itunes'
                                        ? (searchResults as ItunesResult[]).map(renderItunesResult)
                                        : (searchResults as PodindexResult[]).map(renderPodindexResult)}
                                </>
                            )}

                            {/* Empty state */}
                            {!searchMutation.isPending && searchResults.length === 0 && searchQuery.trim() && (
                                <View
                                    style={{
                                        alignItems: 'center',
                                        justifyContent: 'center',
                                        paddingVertical: 40,
                                    }}
                                >
                                    <Ionicons name="search-outline" size={48} color={styles.gray} />
                                    <ThemedText
                                        style={{
                                            fontSize: 16,
                                            color: styles.gray,
                                            marginTop: 12,
                                            textAlign: 'center',
                                        }}
                                    >
                                        {t('no-results', {
                                            defaultValue: 'Keine Ergebnisse gefunden',
                                        })}
                                    </ThemedText>
                                </View>
                            )}

                            {/* Initial state */}
                            {!searchMutation.isPending && searchResults.length === 0 && !searchQuery.trim() && (
                                <View
                                    style={{
                                        alignItems: 'center',
                                        justifyContent: 'center',
                                        paddingVertical: 40,
                                    }}
                                >
                                    <Ionicons
                                        name={activeTab === 'itunes' ? 'logo-apple' : 'search'}
                                        size={48}
                                        color={styles.gray}
                                    />
                                    <ThemedText
                                        style={{
                                            fontSize: 16,
                                            color: styles.gray,
                                            marginTop: 12,
                                            textAlign: 'center',
                                        }}
                                    >
                                        {activeTab === 'itunes'
                                            ? t('itunes-search-hint', {
                                                  defaultValue:
                                                      'Suche im iTunes Podcast-Verzeichnis',
                                              })
                                            : t('podindex-search-hint', {
                                                  defaultValue:
                                                      'Suche im Podcast Index Verzeichnis',
                                              })}
                                    </ThemedText>
                                </View>
                            )}
                        </>
                    )}

                    {/* RSS Feed */}
                    {activeTab === 'rss' && (
                        <View>
                            <ThemedText
                                style={{
                                    fontSize: 14,
                                    color: styles.gray,
                                    marginBottom: 12,
                                }}
                            >
                                {t('rss-feed-description', {
                                    defaultValue:
                                        'Füge einen Podcast direkt über seine RSS-Feed-URL hinzu.',
                                })}
                            </ThemedText>
                            <View
                                style={{
                                    backgroundColor: styles.darkColor,
                                    borderRadius: 12,
                                    padding: 16,
                                }}
                            >
                                <ThemedText
                                    style={{
                                        fontSize: 14,
                                        color: styles.gray,
                                        marginBottom: 8,
                                    }}
                                >
                                    {t('rss-url-label', { defaultValue: 'RSS-Feed-URL' })}
                                </ThemedText>
                                <TextInput
                                    style={{
                                        backgroundColor: 'rgba(255,255,255,0.05)',
                                        borderRadius: 8,
                                        padding: 14,
                                        color: styles.white,
                                        fontSize: 15,
                                        marginBottom: 16,
                                    }}
                                    placeholder="https://example.com/podcast/feed.xml"
                                    placeholderTextColor={styles.gray}
                                    value={rssUrl}
                                    onChangeText={setRssUrl}
                                    autoCapitalize="none"
                                    autoCorrect={false}
                                    keyboardType="url"
                                />
                                <TouchableOpacity
                                    style={{
                                        backgroundColor: styles.accentColor,
                                        borderRadius: 8,
                                        padding: 14,
                                        alignItems: 'center',
                                        flexDirection: 'row',
                                        justifyContent: 'center',
                                        gap: 8,
                                    }}
                                    onPress={handleAddFromRss}
                                    disabled={isAdding || !rssUrl.trim()}
                                >
                                    {isAdding ? (
                                        <ActivityIndicator size="small" color={styles.white} />
                                    ) : (
                                        <>
                                            <Ionicons name="add-circle" size={20} color={styles.white} />
                                            <ThemedText
                                                style={{
                                                    color: styles.white,
                                                    fontSize: 16,
                                                    fontWeight: '600',
                                                }}
                                            >
                                                {t('add-podcast-button', {
                                                    defaultValue: 'Podcast hinzufügen',
                                                })}
                                            </ThemedText>
                                        </>
                                    )}
                                </TouchableOpacity>
                            </View>
                        </View>
                    )}

                    {/* OPML Import */}
                    {activeTab === 'opml' && (
                        <View>
                            <ThemedText
                                style={{
                                    fontSize: 14,
                                    color: styles.gray,
                                    marginBottom: 12,
                                }}
                            >
                                {t('opml-description', {
                                    defaultValue:
                                        'Importiere mehrere Podcasts aus einer OPML-Datei. Diese Dateien können aus anderen Podcast-Apps exportiert werden.',
                                })}
                            </ThemedText>
                            <View
                                style={{
                                    backgroundColor: styles.darkColor,
                                    borderRadius: 12,
                                    padding: 24,
                                    alignItems: 'center',
                                }}
                            >
                                <Ionicons
                                    name="document-text-outline"
                                    size={64}
                                    color={styles.gray}
                                />
                                <ThemedText
                                    style={{
                                        fontSize: 16,
                                        color: styles.white,
                                        marginTop: 16,
                                        marginBottom: 8,
                                        textAlign: 'center',
                                    }}
                                >
                                    {t('opml-select-file', {
                                        defaultValue: 'Wähle eine OPML-Datei',
                                    })}
                                </ThemedText>
                                <ThemedText
                                    style={{
                                        fontSize: 13,
                                        color: styles.gray,
                                        marginBottom: 20,
                                        textAlign: 'center',
                                    }}
                                >
                                    {t('opml-file-types', {
                                        defaultValue: 'Unterstützt: .opml, .xml',
                                    })}
                                </ThemedText>
                                <TouchableOpacity
                                    style={{
                                        backgroundColor: styles.accentColor,
                                        borderRadius: 8,
                                        paddingVertical: 14,
                                        paddingHorizontal: 24,
                                        flexDirection: 'row',
                                        alignItems: 'center',
                                        gap: 8,
                                    }}
                                    onPress={handleImportOpml}
                                    disabled={isAdding}
                                >
                                    {isAdding ? (
                                        <ActivityIndicator size="small" color={styles.white} />
                                    ) : (
                                        <>
                                            <Ionicons name="folder-open" size={20} color={styles.white} />
                                            <ThemedText
                                                style={{
                                                    color: styles.white,
                                                    fontSize: 16,
                                                    fontWeight: '600',
                                                }}
                                            >
                                                {t('select-file', { defaultValue: 'Datei auswählen' })}
                                            </ThemedText>
                                        </>
                                    )}
                                </TouchableOpacity>
                            </View>

                            {/* OPML Tips */}
                            <View
                                style={{
                                    backgroundColor: styles.darkColor,
                                    borderRadius: 12,
                                    padding: 16,
                                    marginTop: 16,
                                }}
                            >
                                <View
                                    style={{
                                        flexDirection: 'row',
                                        alignItems: 'center',
                                        gap: 8,
                                        marginBottom: 12,
                                    }}
                                >
                                    <Ionicons name="information-circle" size={20} color={styles.accentColor} />
                                    <ThemedText
                                        style={{
                                            fontSize: 15,
                                            color: styles.white,
                                            fontWeight: '600',
                                        }}
                                    >
                                        {t('opml-tip-title', { defaultValue: 'Tipp' })}
                                    </ThemedText>
                                </View>
                                <ThemedText
                                    style={{
                                        fontSize: 13,
                                        color: styles.gray,
                                        lineHeight: 20,
                                    }}
                                >
                                    {t('opml-tip-content', {
                                        defaultValue:
                                            'Die meisten Podcast-Apps bieten eine Export-Funktion für OPML-Dateien an. Suche in den Einstellungen deiner aktuellen App nach "Export" oder "OPML".',
                                    })}
                                </ThemedText>
                            </View>
                        </View>
                    )}
                </ScrollView>
            </KeyboardAvoidingView>
        </SafeAreaView>
    );
};

export default AddPodcastScreen;

