import {View, Text, Pressable, Image, StyleSheet, ScrollView} from "react-native";
import {Ionicons} from "@expo/vector-icons";
import React from "react";
import {styles as appStyles} from "@/styles/styles";
import {$api} from "@/client";

export default function () {
    const podcasts = $api.useQuery('get', '/api/v1/podcasts')

        return (
            <ScrollView style={{marginLeft: 20, marginRight: 20, marginTop: 20, flex: 1}}>
                {
                    podcasts.data?.map((p)=>
                        <Pressable
                            key={p.id}
                            style={styles.episodeCard}
                        >
                            <Image
                                source={{ uri: p.image_url }}
                                style={styles.episodeImage}
                            />
                            <View style={styles.episodeInfo}>
                                <Text style={styles.episodeName} numberOfLines={2}>
                                    {p.name}
                                </Text>
                                <Text style={styles.podcastName} numberOfLines={1}>
                                    {p.summary}
                                </Text>
                                <View style={styles.episodeMeta}>
                                    <Ionicons name="download" size={12} color="#4ade80" />
                                    <Text style={styles.metaDivider}>•</Text>
                                </View>
                            </View>
                            <Pressable
                                style={styles.deleteButton}
                            >
                                <Ionicons name="trash-outline" size={20} color="#ef4444" />
                            </Pressable>
                        </Pressable>)
                }
            </ScrollView>
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
        paddingBottom: 180,
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