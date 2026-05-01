import { View, Pressable, Text } from 'react-native';
import { useTranslation } from 'react-i18next';
import { MaterialCommunityIcons } from '@expo/vector-icons';
import { styles } from '@/styles/styles';
import { useStore } from '@/store/store';
import { useCastControls } from '@/hooks/useCastSession';

export const CastingPill = () => {
    const { t } = useTranslation();
    const castSession = useStore((s) => s.castSession);
    const castDeviceName = useStore((s) => s.castDeviceName);
    const { stopCasting, isPending } = useCastControls();

    if (!castSession) return null;

    return (
        <View style={{
            flexDirection: 'row',
            alignItems: 'center',
            backgroundColor: 'rgba(230,154,19,0.15)',
            borderColor: styles.accentColor,
            borderWidth: 1,
            borderRadius: 999,
            paddingVertical: 6,
            paddingLeft: 12,
            paddingRight: 6,
            alignSelf: 'center',
            gap: 8,
            marginVertical: 8,
        }}>
            <MaterialCommunityIcons name="cast-connected" size={16} color={styles.accentColor} />
            <Text style={{ color: styles.accentColor, fontSize: 12, fontWeight: '600' }} numberOfLines={1}>
                {t('cast-casting-on', {
                    defaultValue: 'Casting on {{name}}',
                    name: castDeviceName ?? t('cast-device', { defaultValue: 'device' }),
                })}
            </Text>
            <Pressable
                onPress={stopCasting}
                disabled={isPending}
                style={{
                    backgroundColor: styles.accentColor,
                    borderRadius: 999,
                    paddingVertical: 4,
                    paddingHorizontal: 10,
                    opacity: isPending ? 0.6 : 1,
                }}
            >
                <Text style={{ color: styles.white, fontSize: 11, fontWeight: '700' }}>
                    {t('cast-stop', { defaultValue: 'Stop' })}
                </Text>
            </Pressable>
        </View>
    );
};
