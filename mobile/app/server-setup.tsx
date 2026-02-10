import { useState } from 'react';
import { View, TextInput, TouchableOpacity, ActivityIndicator, KeyboardAvoidingView, Platform } from 'react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
import { useTranslation } from 'react-i18next';
import { useRouter } from 'expo-router';
import { ThemedText } from '@/components/ThemedText';
import { ThemedView } from '@/components/ThemedView';
import { styles } from '@/styles/styles';
import { useStore } from '@/store/store';
import { validatePodFetchServer } from '@/client';
import { IconSymbol } from '@/components/ui/IconSymbol';

const ServerSetupScreen = () => {
    const { t } = useTranslation();
    const router = useRouter();
    const setServerUrl = useStore((state) => state.setServerUrl);

    const [url, setUrl] = useState('');
    const [isValidating, setIsValidating] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const normalizeUrl = (inputUrl: string): string => {
        let normalized = inputUrl.trim();
        if (!normalized.startsWith('http://') && !normalized.startsWith('https://')) {
            normalized = 'http://' + normalized;
        }
        return normalized.replace(/\/$/, '');
    };

    const handleConnect = async () => {
        if (!url.trim()) {
            setError(t('server-url-required'));
            return;
        }

        setIsValidating(true);
        setError(null);

        try {
            const isValid = await validatePodFetchServer(url);

            if (isValid) {
                const normalizedUrl = normalizeUrl(url);
                setServerUrl(normalizedUrl);
                router.replace('/(tabs)');
            } else {
                setError(t('server-not-found'));
            }
        } catch (err) {
            setError(t('connection-failed'));
        } finally {
            setIsValidating(false);
        }
    };

    return (
        <SafeAreaView style={{ flex: 1, backgroundColor: styles.lightDarkColor }}>
            <KeyboardAvoidingView
                behavior={Platform.OS === 'ios' ? 'padding' : 'height'}
                style={{ flex: 1 }}
            >
                <ThemedView style={{
                    flex: 1,
                    backgroundColor: styles.lightDarkColor,
                    paddingHorizontal: 20,
                    justifyContent: 'center',
                }}>
                    {/* Logo/Icon */}
                    <View style={{ alignItems: 'center', marginBottom: 40 }}>
                        <IconSymbol
                            name="antenna.radiowaves.left.and.right"
                            size={80}
                            color={styles.accentColor}
                        />
                        <ThemedText style={{
                            fontSize: 28,
                            fontWeight: 'bold',
                            marginTop: 20,
                            color: styles.white
                        }}>
                            PodFetch
                        </ThemedText>
                        <ThemedText style={{
                            fontSize: 16,
                            color: styles.gray,
                            marginTop: 8,
                            textAlign: 'center'
                        }}>
                            {t('server-setup-description')}
                        </ThemedText>
                    </View>

                    {/* URL Input */}
                    <View style={{ marginBottom: 20 }}>
                        <ThemedText style={{
                            fontSize: 14,
                            color: styles.gray,
                            marginBottom: 8
                        }}>
                            {t('server-url')}
                        </ThemedText>
                        <TextInput
                            style={{
                                backgroundColor: styles.darkColor,
                                borderRadius: 10,
                                padding: 15,
                                fontSize: 16,
                                color: styles.white,
                                borderWidth: 1,
                                borderColor: error ? '#ff4444' : styles.lightgray,
                            }}
                            placeholder={t('server-url-placeholder')}
                            placeholderTextColor={styles.gray}
                            value={url}
                            onChangeText={(text) => {
                                setUrl(text);
                                setError(null);
                            }}
                            autoCapitalize="none"
                            autoCorrect={false}
                            keyboardType="url"
                        />
                        {error && (
                            <ThemedText style={{
                                color: '#ff4444',
                                fontSize: 14,
                                marginTop: 8
                            }}>
                                {error}
                            </ThemedText>
                        )}
                    </View>

                    {/* Connect Button */}
                    <TouchableOpacity
                        style={{
                            backgroundColor: styles.accentColor,
                            borderRadius: 10,
                            padding: 15,
                            alignItems: 'center',
                            opacity: isValidating ? 0.7 : 1,
                        }}
                        onPress={handleConnect}
                        disabled={isValidating}
                    >
                        {isValidating ? (
                            <ActivityIndicator color={styles.white} />
                        ) : (
                            <ThemedText style={{
                                color: styles.white,
                                fontSize: 16,
                                fontWeight: '600'
                            }}>
                                {t('connect')}
                            </ThemedText>
                        )}
                    </TouchableOpacity>

                    {/* Help Text */}
                    <ThemedText style={{
                        fontSize: 12,
                        color: styles.gray,
                        textAlign: 'center',
                        marginTop: 20
                    }}>
                        {t('server-setup-help')}
                    </ThemedText>
                </ThemedView>
            </KeyboardAvoidingView>
        </SafeAreaView>
    );
};

export default ServerSetupScreen;
