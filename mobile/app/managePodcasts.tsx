import {
    View,
    Text,
    Pressable,
    Image,
    StyleSheet,
    ScrollView, Alert,
} from "react-native";
import { Ionicons } from "@expo/vector-icons";
import React, { useCallback, useState } from "react";
import { WebView, WebViewMessageEvent } from "react-native-webview";
import { styles as appStyles } from "@/styles/styles";
import { $api } from "@/client";

const SUMMARY_BASE_HTML = `
  <html>
    <head>
      <meta name="viewport" content="width=device-width, initial-scale=1, maximum-scale=1">
      <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
          font-family: -apple-system, system-ui, sans-serif;
          font-size: 13px;
          line-height: 1.5;
          color: rgba(255,255,255,0.55);
          background-color: transparent;
          overflow: hidden;
        }
        a { color: ${appStyles.accentColor ?? "#6d9eff"}; }
        p { margin-bottom: 6px; }
        p:last-child { margin-bottom: 0; }
        img { max-width: 100%; border-radius: 6px; }
      </style>
    </head>
    <body>
      %%CONTENT%%
      <script>
        function postHeight() {
          const height = document.body.scrollHeight;
          window.ReactNativeWebView.postMessage(JSON.stringify({ height }));
        }
        window.addEventListener('load', postHeight);
        new ResizeObserver(postHeight).observe(document.body);
      </script>
    </body>
  </html>
`;

function HtmlSummary({ html }: { html: string }) {
    const [height, setHeight] = useState(40);

    const onMessage = useCallback((e: WebViewMessageEvent) => {
        try {
            const data = JSON.parse(e.nativeEvent.data);
            if (data.height && typeof data.height === "number") {
                setHeight(Math.min(data.height, 300));
            }
        } catch {}
    }, []);

    const source = {
        html: SUMMARY_BASE_HTML.replace("%%CONTENT%%", html),
    };

    return (
        <WebView
            source={source}
            style={[styles.webview, { height }]}
            scrollEnabled={false}
            showsVerticalScrollIndicator={false}
            originWhitelist={["*"]}
            onMessage={onMessage}
            javaScriptEnabled
            transparent
        />
    );
}

export default function () {
    const podcasts = $api.useQuery("get", "/api/v1/podcasts");
    const deletePodcast = $api.useMutation("delete", "/api/v1/podcasts/{id}")

    return (
        <ScrollView
            style={styles.container}
            contentContainerStyle={styles.contentContainer}
        >
            <Text style={styles.headerTitle}>Podcasts</Text>
            <Text style={styles.headerSubtitle}>
                {podcasts.data?.length ?? 0} podcast
                {podcasts.data?.length === 1 ? "" : "s"} added
            </Text>

            {podcasts.data?.map((p) => (
                <View key={p.id} style={styles.card}>
                    <View style={styles.cardHeader}>
                        <Image
                            source={{ uri: p.image_url }}
                            style={styles.cardImage}
                        />
                        <View style={styles.cardHeaderInfo}>
                            <Text style={styles.cardTitle} numberOfLines={2}>
                                {p.name}
                            </Text>
                            <Text style={styles.cardAuthor} numberOfLines={2}>
                                {p.author}
                            </Text>
                        </View>
                    </View>

                    {!!p.summary && (
                        <View style={styles.summaryContainer}>
                            <HtmlSummary html={p.summary} />
                        </View>
                    )}

                    {p.keywords && p.keywords.length > 0 && (
                        <View style={styles.keywordsContainer}>
                            <View style={styles.keywordsRow}>
                                {p.keywords.split(',').map((keyword, i) => (
                                    <View key={i} style={styles.keywordBadge}>
                                        <Text style={styles.keywordText}>
                                            {keyword}
                                        </Text>
                                    </View>
                                ))}
                            </View>
                        </View>
                    )}

                    <View style={styles.cardFooter}>
                        <View style={styles.idContainer}>
                            <Ionicons
                                name="finger-print-outline"
                                size={14}
                                color="rgba(255,255,255,0.4)"
                            />
                            <Text style={styles.idText}>
                                {p.id}…
                            </Text>
                        </View>
                        <Pressable style={styles.deleteButton} hitSlop={8} onPress={()=>{
                            Alert.alert(
                                'Podcast löschen',
                                `Möchtest du "${p.name}" wirklich löschen?`,
                                [
                                    { text: 'Abbrechen', style: 'cancel' },
                                    {
                                        text: 'Löschen',
                                        style: 'destructive',
                                        onPress: () => {
                                            deletePodcast.mutateAsync({
                                                params: {
                                                    path: {
                                                        id: p.id
                                                    }
                                                },
                                                body: {
                                                    delete_files: false
                                                }
                                            }).then(()=>{
                                                podcasts.refetch();
                                            })
                                        }
                                    }
                                ]
                            );
                        }}>
                            <Ionicons
                                name="trash-outline"
                                size={18}
                                color="#ef4444"
                            />
                        </Pressable>
                    </View>
                </View>
            ))}
        </ScrollView>
    );
}

const styles = StyleSheet.create({
    container: {
        flex: 1,
    },
    contentContainer: {
        padding: 20,
        paddingBottom: 100,
    },
    headerTitle: {
        fontSize: 26,
        fontWeight: "bold",
        color: "#fff",
    },
    headerSubtitle: {
        fontSize: 14,
        color: "rgba(255,255,255,0.5)",
        marginTop: 4,
        marginBottom: 20,
    },
    card: {
        backgroundColor: "rgba(255,255,255,0.06)",
        borderRadius: 16,
        borderWidth: 1,
        borderColor: "rgba(255,255,255,0.08)",
        padding: 16,
        marginBottom: 14,
    },
    cardHeader: {
        flexDirection: "row",
        alignItems: "flex-start",
    },
    cardImage: {
        width: 72,
        height: 72,
        borderRadius: 12,
        backgroundColor: "rgba(255,255,255,0.1)",
    },
    cardHeaderInfo: {
        flex: 1,
        marginLeft: 14,
        justifyContent: "center",
    },
    cardTitle: {
        fontSize: 17,
        fontWeight: "700",
        color: "#fff",
        lineHeight: 22,
    },
    cardAuthor: {
      fontSize: 13,
        color: "rgba(255,255,255,0.5)",
    },
    summaryContainer: {
        marginTop: 12,
        paddingTop: 12,
        borderTopWidth: 1,
        borderTopColor: "rgba(255,255,255,0.06)",
    },
    webview: {
        backgroundColor: "transparent",
        width: "100%",
    },
    keywordsContainer: {
        marginTop: 12,
        paddingTop: 12,
        borderTopWidth: 1,
        borderTopColor: "rgba(255,255,255,0.06)",
    },
    keywordsRow: {
        flexDirection: "row",
        flexWrap: "wrap",
        gap: 6,
    },
    keywordBadge: {
        backgroundColor: "rgba(255,255,255,0.08)",
        borderRadius: 20,
        paddingHorizontal: 10,
        paddingVertical: 4,
    },
    keywordText: {
        fontSize: 12,
        color: "rgba(255,255,255,0.6)",
        fontWeight: "500",
    },
    cardFooter: {
        flexDirection: "row",
        justifyContent: "space-between",
        alignItems: "center",
        marginTop: 12,
        paddingTop: 12,
        borderTopWidth: 1,
        borderTopColor: "rgba(255,255,255,0.06)",
    },
    idContainer: {
        flexDirection: "row",
        alignItems: "center",
        gap: 6,
    },
    idText: {
        fontSize: 12,
        color: "rgba(255,255,255,0.35)",
        fontFamily: "monospace",
    },
    deleteButton: {
        padding: 6,
        borderRadius: 8,
        backgroundColor: "rgba(239, 68, 68, 0.1)",
    },
});