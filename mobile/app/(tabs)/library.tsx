import {Image, StyleSheet} from "react-native";
import {ThemedView} from "@/components/ThemedView";
import {ThemedText} from "@/components/ThemedText";
import ParallaxScrollView from "@/components/ParallaxScrollView";


const styles = StyleSheet.create({
    titleContainer: {
        flexDirection: 'row',
        alignItems: 'center',
        gap: 8,
    },
    stepContainer: {
        gap: 8,
        marginBottom: 8,
    },
    reactLogo: {
        height: 178,
        width: 290,
        bottom: 0,
        left: 0,
        position: 'absolute',
    },
});


export default function Library() {
    return <ParallaxScrollView
        headerBackgroundColor={{ light: '#A1CEDC', dark: '#1D3D47' }}
        headerImage={
            <Image
                source={require('@/assets/images/partial-react-logo.png')}
                style={styles.reactLogo}
            />
        }>
        <ThemedView style={styles.titleContainer}>
            <ThemedText type="title">Library!</ThemedText>
        </ThemedView>
    </ParallaxScrollView>
}
