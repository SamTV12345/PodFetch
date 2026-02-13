import { useState, useCallback } from 'react';
import { View, TextInput, TouchableOpacity, ActivityIndicator, KeyboardAvoidingView, Platform } from 'react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
import { useTranslation } from 'react-i18next';
import { useRouter } from 'expo-router';
import * as WebBrowser from 'expo-web-browser';
import * as Linking from 'expo-linking';
import { ThemedText } from '@/components/ThemedText';
import { ThemedView } from '@/components/ThemedView';
import { styles } from '@/styles/styles';
import { useStore } from '@/store/store';
import { validatePodFetchServer, validateBasicAuth, exchangeOidcCode, fetchUserProfile } from '@/client';
import { IconSymbol } from '@/components/ui/IconSymbol';
import { PodFetchLogo } from '@/components/PodFetchLogo';
import { components } from '@/schema';

type AuthStep = 'url' | 'auth-choice' | 'basic-auth' | 'oidc';

const ServerSetupScreen = () => {
    const { t } = useTranslation();
    const router = useRouter();
    const {
        setServerUrl,
        setServerConfig,
        setAuthType,
        setBasicAuthUsername,
        setBasicAuthPassword,
        setOidcAccessToken,
        setOidcRefreshToken,
        setOidcTokenExpiry,
        setUserProfile
    } = useStore();

    const [url, setUrl] = useState('');
    const [username, setUsername] = useState('');
    const [password, setPassword] = useState('');
    const [isValidating, setIsValidating] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [showPassword, setShowPassword] = useState(false);
    const [authStep, setAuthStep] = useState<AuthStep>('url');
    const [serverConfig, setLocalServerConfig] = useState<components["schemas"]["ConfigModel"] | null>(null);
    const [normalizedUrl, setNormalizedUrl] = useState('');

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
            const result = await validatePodFetchServer(url);

            if (result.success) {
                const normalized = normalizeUrl(url);
                setNormalizedUrl(normalized);
                setLocalServerConfig(result.config);

                if (result.config.basicAuth && result.config.oidcConfigured) {
                    setAuthStep('auth-choice');
                } else if (result.config.basicAuth) {
                    setAuthStep('basic-auth');
                } else if (result.config.oidcConfigured) {
                    setAuthStep('oidc');
                } else {
                    // No auth required - connect directly
                    setServerUrl(normalized);
                    setServerConfig(result.config);
                    setAuthType('none');
                    router.replace('/(tabs)');
                }
            } else {
                setError(t('server-not-found'));
            }
        } catch (err) {
            setError(t('connection-failed'));
        } finally {
            setIsValidating(false);
        }
    };

    const handleBasicAuth = async () => {
        if (!username.trim() || !password.trim()) {
            setError(t('auth-failed'));
            return;
        }

        setIsValidating(true);
        setError(null);

        try {
            const isValid = await validateBasicAuth(normalizedUrl, username, password);

            if (isValid) {
                // Fetch user profile to get API key
                const userProfile = await fetchUserProfile(normalizedUrl, username, password);

                setServerUrl(normalizedUrl);
                setServerConfig(serverConfig);
                setAuthType('basic');
                setBasicAuthUsername(username);
                setBasicAuthPassword(password);

                if (userProfile) {
                    setUserProfile(userProfile);
                }

                router.replace('/(tabs)');
            } else {
                setError(t('auth-failed'));
            }
        } catch (err) {
            setError(t('auth-failed'));
        } finally {
            setIsValidating(false);
        }
    };

    const handleOidcLogin = useCallback(async () => {
        if (!serverConfig?.oidcConfig) {
            setError(t('oidc-failed'));
            return;
        }

        setIsValidating(true);
        setError(null);

        try {
            const oidcConfig = serverConfig.oidcConfig;
            const redirectUri = Linking.createURL('auth/callback');
            const scope = oidcConfig.scope.includes('offline_access')
                ? oidcConfig.scope
                : `${oidcConfig.scope} offline_access`;

            const authUrl = new URL(`${oidcConfig.authority}/protocol/openid-connect/auth`);
            authUrl.searchParams.set('client_id', oidcConfig.clientId);
            authUrl.searchParams.set('redirect_uri', redirectUri);
            authUrl.searchParams.set('response_type', 'code');
            authUrl.searchParams.set('scope', scope);
            authUrl.searchParams.set('prompt', 'consent'); // Ensure we get refresh token

            // Open browser for authentication
            const result = await WebBrowser.openAuthSessionAsync(
                authUrl.toString(),
                redirectUri
            );

            if (result.type === 'success' && result.url) {
                // Parse the callback URL to extract the authorization code
                const callbackUrl = new URL(result.url);
                const code = callbackUrl.searchParams.get('code');

                if (code) {
                    // Exchange code for tokens
                    const tokenEndpoint = `${oidcConfig.authority}/protocol/openid-connect/token`;
                    const tokenResult = await exchangeOidcCode(
                        tokenEndpoint,
                        code,
                        oidcConfig.clientId,
                        redirectUri
                    );

                    if (tokenResult) {
                        setServerUrl(normalizedUrl);
                        setServerConfig(serverConfig);
                        setAuthType('oidc');
                        setOidcAccessToken(tokenResult.access_token);
                        if (tokenResult.refresh_token) {
                            setOidcRefreshToken(tokenResult.refresh_token);
                        }
                        if (tokenResult.expires_in) {
                            setOidcTokenExpiry(Date.now() + tokenResult.expires_in * 1000);
                        }
                        router.replace('/(tabs)');
                        return;
                    }
                }
            }

            setError(t('oidc-failed'));
        } catch (err) {
            console.error('OIDC login error:', err);
            setError(t('oidc-failed'));
        } finally {
            setIsValidating(false);
        }
    }, [serverConfig, normalizedUrl, t, router, setServerUrl, setServerConfig, setAuthType, setOidcAccessToken, setOidcRefreshToken, setOidcTokenExpiry]);

    const handleBack = () => {
        setAuthStep('url');
        setError(null);
        setUsername('');
        setPassword('');
    };

    const renderHeader = () => {
        if (authStep === 'url') return null;

        return (
            <View style={{
                position: 'absolute',
                top: 0,
                left: 0,
                right: 0,
                paddingHorizontal: 20,
                paddingTop: 10,
                zIndex: 10,
            }}>
                <TouchableOpacity
                    style={{
                        flexDirection: 'row',
                        alignItems: 'center',
                        alignSelf: 'flex-start',
                        paddingVertical: 8,
                        paddingRight: 16,
                    }}
                    onPress={handleBack}
                >
                    <IconSymbol name="chevron.left" size={20} color={styles.accentColor} />
                    <ThemedText style={{ color: styles.accentColor, fontSize: 16, marginLeft: 4 }}>
                        {t('back')}
                    </ThemedText>
                </TouchableOpacity>
            </View>
        );
    };

    const renderUrlStep = () => (
        <>
            {/* Logo/Icon */}
            <View style={{ alignItems: 'center', marginBottom: 40 }}>
                <PodFetchLogo size={100} />
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
        </>
    );

    const renderAuthChoice = () => (
        <>

            <View style={{ alignItems: 'center', marginBottom: 40 }}>
                <IconSymbol
                    name="lock.fill"
                    size={60}
                    color={styles.accentColor}
                />
                <ThemedText style={{
                    fontSize: 24,
                    fontWeight: 'bold',
                    marginTop: 20,
                    color: styles.white
                }}>
                    {t('auth-required')}
                </ThemedText>
                <ThemedText style={{
                    fontSize: 14,
                    color: styles.gray,
                    marginTop: 8,
                    textAlign: 'center'
                }}>
                    {t('choose-auth-method')}
                </ThemedText>
            </View>

            {/* Basic Auth Option */}
            <TouchableOpacity
                style={{
                    backgroundColor: styles.darkColor,
                    borderRadius: 10,
                    padding: 20,
                    marginBottom: 15,
                    flexDirection: 'row',
                    alignItems: 'center',
                }}
                onPress={() => setAuthStep('basic-auth')}
            >
                <IconSymbol name="person.fill" size={24} color={styles.white} />
                <View style={{ marginLeft: 15, flex: 1 }}>
                    <ThemedText style={{ fontSize: 16, fontWeight: '600', color: styles.white }}>
                        {t('login')}
                    </ThemedText>
                    <ThemedText style={{ fontSize: 12, color: styles.gray, marginTop: 4 }}>
                        {t('basic-auth-description')}
                    </ThemedText>
                </View>
            </TouchableOpacity>

            {/* OIDC Option */}
            <TouchableOpacity
                style={{
                    backgroundColor: styles.darkColor,
                    borderRadius: 10,
                    padding: 20,
                    flexDirection: 'row',
                    alignItems: 'center',
                }}
                onPress={() => setAuthStep('oidc')}
            >
                <IconSymbol name="key.fill" size={24} color={styles.white} />
                <View style={{ marginLeft: 15, flex: 1 }}>
                    <ThemedText style={{ fontSize: 16, fontWeight: '600', color: styles.white }}>
                        {t('login-with-sso')}
                    </ThemedText>
                    <ThemedText style={{ fontSize: 12, color: styles.gray, marginTop: 4 }}>
                        {t('oidc-auth-description')}
                    </ThemedText>
                </View>
            </TouchableOpacity>
        </>
    );

    const renderBasicAuth = () => (
        <>

            <View style={{ alignItems: 'center', marginBottom: 40 }}>
                <IconSymbol
                    name="person.fill"
                    size={60}
                    color={styles.accentColor}
                />
                <ThemedText style={{
                    fontSize: 24,
                    fontWeight: 'bold',
                    marginTop: 20,
                    color: styles.white
                }}>
                    {t('login')}
                </ThemedText>
                <ThemedText style={{
                    fontSize: 14,
                    color: styles.gray,
                    marginTop: 8,
                    textAlign: 'center'
                }}>
                    {t('basic-auth-description')}
                </ThemedText>
            </View>

            {/* Username Input */}
            <View style={{ marginBottom: 15 }}>
                <ThemedText style={{
                    fontSize: 14,
                    color: styles.gray,
                    marginBottom: 8
                }}>
                    {t('username')}
                </ThemedText>
                <TextInput
                    style={{
                        backgroundColor: styles.darkColor,
                        borderRadius: 10,
                        padding: 15,
                        fontSize: 16,
                        color: styles.white,
                        borderWidth: 1,
                        borderColor: styles.lightgray,
                    }}
                    placeholder={t('username-placeholder')}
                    placeholderTextColor={styles.gray}
                    value={username}
                    onChangeText={(text) => {
                        setUsername(text);
                        setError(null);
                    }}
                    autoCapitalize="none"
                    autoCorrect={false}
                />
            </View>

            {/* Password Input */}
            <View style={{ marginBottom: 20 }}>
                <ThemedText style={{
                    fontSize: 14,
                    color: styles.gray,
                    marginBottom: 8
                }}>
                    {t('password')}
                </ThemedText>
                <View style={{ position: 'relative' }}>
                    <TextInput
                        style={{
                            backgroundColor: styles.darkColor,
                            borderRadius: 10,
                            padding: 15,
                            paddingRight: 50,
                            fontSize: 16,
                            color: styles.white,
                            borderWidth: 1,
                            borderColor: error ? '#ff4444' : styles.lightgray,
                        }}
                        placeholder={t('password-placeholder')}
                        placeholderTextColor={styles.gray}
                        value={password}
                        onChangeText={(text) => {
                            setPassword(text);
                            setError(null);
                        }}
                        secureTextEntry={!showPassword}
                        autoCapitalize="none"
                        autoCorrect={false}
                    />
                    <TouchableOpacity
                        style={{
                            position: 'absolute',
                            right: 15,
                            top: 0,
                            bottom: 0,
                            justifyContent: 'center',
                        }}
                        onPress={() => setShowPassword(!showPassword)}
                    >
                        <IconSymbol
                            name={showPassword ? "eye.slash.fill" : "eye.fill"}
                            size={22}
                            color={styles.gray}
                        />
                    </TouchableOpacity>
                </View>
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

            {/* Login Button */}
            <TouchableOpacity
                style={{
                    backgroundColor: styles.accentColor,
                    borderRadius: 10,
                    padding: 15,
                    alignItems: 'center',
                    opacity: isValidating ? 0.7 : 1,
                }}
                onPress={handleBasicAuth}
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
                        {t('login')}
                    </ThemedText>
                )}
            </TouchableOpacity>
        </>
    );

    const renderOidcAuth = () => (
        <>

            <View style={{ alignItems: 'center', marginBottom: 40 }}>
                <IconSymbol
                    name="key.fill"
                    size={60}
                    color={styles.accentColor}
                />
                <ThemedText style={{
                    fontSize: 24,
                    fontWeight: 'bold',
                    marginTop: 20,
                    color: styles.white
                }}>
                    {t('login-with-sso')}
                </ThemedText>
                <ThemedText style={{
                    fontSize: 14,
                    color: styles.gray,
                    marginTop: 8,
                    textAlign: 'center'
                }}>
                    {t('oidc-auth-description')}
                </ThemedText>
            </View>

            {error && (
                <ThemedText style={{
                    color: '#ff4444',
                    fontSize: 14,
                    marginBottom: 20,
                    textAlign: 'center'
                }}>
                    {error}
                </ThemedText>
            )}

            {/* SSO Login Button */}
            <TouchableOpacity
                style={{
                    backgroundColor: styles.accentColor,
                    borderRadius: 10,
                    padding: 15,
                    alignItems: 'center',
                    opacity: isValidating ? 0.7 : 1,
                }}
                onPress={handleOidcLogin}
                disabled={isValidating}
            >
                {isValidating ? (
                    <View style={{ flexDirection: 'row', alignItems: 'center' }}>
                        <ActivityIndicator color={styles.white} />
                        <ThemedText style={{
                            color: styles.white,
                            fontSize: 16,
                            fontWeight: '600',
                            marginLeft: 10
                        }}>
                            {t('logging-in')}
                        </ThemedText>
                    </View>
                ) : (
                    <ThemedText style={{
                        color: styles.white,
                        fontSize: 16,
                        fontWeight: '600'
                    }}>
                        {t('login-with-sso')}
                    </ThemedText>
                )}
            </TouchableOpacity>
        </>
    );

    const renderContent = () => {
        switch (authStep) {
            case 'url':
                return renderUrlStep();
            case 'auth-choice':
                return renderAuthChoice();
            case 'basic-auth':
                return renderBasicAuth();
            case 'oidc':
                return renderOidcAuth();
        }
    };

    return (
        <SafeAreaView style={{ flex: 1, backgroundColor: styles.lightDarkColor }}>
            <KeyboardAvoidingView
                behavior={Platform.OS === 'ios' ? 'padding' : 'height'}
                style={{ flex: 1 }}
            >
                {/* Header with back button */}
                {renderHeader()}

                <ThemedView style={{
                    flex: 1,
                    backgroundColor: styles.lightDarkColor,
                    paddingHorizontal: 20,
                    justifyContent: 'center',
                }}>
                    {renderContent()}
                </ThemedView>
            </KeyboardAvoidingView>
        </SafeAreaView>
    );
};

export default ServerSetupScreen;
