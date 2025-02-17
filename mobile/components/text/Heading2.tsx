import {ThemedText} from "@/components/ThemedText";
import {Pressable, StyleProp, Text, View, ViewStyle} from "react-native";
import {useTranslation} from "react-i18next";

export default function ({ children, more, onMore, styles }: { children: string, more?: boolean, onMore?: ()=>void, styles?:  StyleProp<ViewStyle> }) {
    const {t} = useTranslation()

    return (
        <View style={[{
            flexDirection: 'row',
            alignItems: 'center',
            paddingLeft: 20,
            paddingBottom: 5,
        }, styles]}>
        <ThemedText style={{color: 'white', fontSize: 20, fontWeight: 'bold', paddingBottom: 5}}>{children}</ThemedText>
    {more && <Pressable onPress={onMore} style={{
        backgroundColor: 'rgba(217, 217, 217, 0.3)',
        paddingLeft: 10,
        paddingRight: 10,
        borderRadius: 10,
        marginLeft: 10
    }}><ThemedText style={{color: 'white', fontSize: 10}}>{t('more')}</ThemedText>
    </Pressable>}
        </View>
    )
}
