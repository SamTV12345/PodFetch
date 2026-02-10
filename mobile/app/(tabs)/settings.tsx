import { View, TouchableOpacity, Alert } from 'react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
import { useTranslation } from 'react-i18next';
import { useRouter } from 'expo-router';
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
