import React, { useEffect, useRef } from 'react';
import { View, Animated, StyleSheet, useWindowDimensions } from 'react-native';
import Svg, { Path, Defs, LinearGradient, Stop, G } from 'react-native-svg';
import { ThemedText } from '@/components/ThemedText';

const AnimatedPath = Animated.createAnimatedComponent(Path);

type SplashScreenProps = {
    onFinish?: () => void;
};

/**
 * Animierter Splashscreen mit PodFetch Logo
 * Zeigt das Logo mit pulsierenden Radiowellen
 */
export const SplashScreen: React.FC<SplashScreenProps> = ({ onFinish }) => {
    const { width: screenWidth, height: screenHeight } = useWindowDimensions();

    // Animations
    const fadeAnim = useRef(new Animated.Value(0)).current;
    const scaleAnim = useRef(new Animated.Value(0.8)).current;
    const wave1Opacity = useRef(new Animated.Value(0)).current;
    const wave2Opacity = useRef(new Animated.Value(0)).current;
    const textFade = useRef(new Animated.Value(0)).current;

    const logoSize = Math.min(screenWidth * 0.5, 200);

    useEffect(() => {
        // Logo einblenden und skalieren
        Animated.parallel([
            Animated.timing(fadeAnim, {
                toValue: 1,
                duration: 500,
                useNativeDriver: true,
            }),
            Animated.spring(scaleAnim, {
                toValue: 1,
                friction: 8,
                tension: 40,
                useNativeDriver: true,
            }),
        ]).start();

        // Wellen pulsieren lassen
        const pulseWaves = () => {
            Animated.loop(
                Animated.sequence([
                    Animated.parallel([
                        Animated.timing(wave1Opacity, {
                            toValue: 1,
                            duration: 600,
                            useNativeDriver: true,
                        }),
                        Animated.timing(wave2Opacity, {
                            toValue: 0.6,
                            duration: 600,
                            delay: 200,
                            useNativeDriver: true,
                        }),
                    ]),
                    Animated.parallel([
                        Animated.timing(wave1Opacity, {
                            toValue: 0.4,
                            duration: 600,
                            useNativeDriver: true,
                        }),
                        Animated.timing(wave2Opacity, {
                            toValue: 1,
                            duration: 600,
                            useNativeDriver: true,
                        }),
                    ]),
                ])
            ).start();
        };

        // Text einblenden
        setTimeout(() => {
            Animated.timing(textFade, {
                toValue: 1,
                duration: 400,
                useNativeDriver: true,
            }).start();
        }, 300);

        pulseWaves();

        // Callback nach Animation (optional)
        if (onFinish) {
            setTimeout(onFinish, 2000);
        }
    }, []);

    return (
        <View style={styles.container}>
            {/* Hintergrund mit Sternen-Effekt */}
            <View style={styles.starsContainer}>
                {[...Array(20)].map((_, i) => (
                    <View
                        key={i}
                        style={[
                            styles.star,
                            {
                                left: `${Math.random() * 100}%`,
                                top: `${Math.random() * 100}%`,
                                opacity: Math.random() * 0.5 + 0.2,
                                width: Math.random() * 3 + 1,
                                height: Math.random() * 3 + 1,
                            },
                        ]}
                    />
                ))}
            </View>

            {/* Logo Container */}
            <Animated.View
                style={[
                    styles.logoContainer,
                    {
                        opacity: fadeAnim,
                        transform: [{ scale: scaleAnim }],
                    },
                ]}
            >
                <Svg width={logoSize} height={logoSize} viewBox="0 0 100 100">
                    <Defs>
                        <LinearGradient id="splashLogoGradient" x1="0%" y1="0%" x2="100%" y2="100%">
                            <Stop offset="0%" stopColor="#8B5CF6" />
                            <Stop offset="30%" stopColor="#A855F7" />
                            <Stop offset="60%" stopColor="#D946EF" />
                            <Stop offset="100%" stopColor="#F59E0B" />
                        </LinearGradient>
                        <LinearGradient id="splashWaveGradient" x1="0%" y1="0%" x2="0%" y2="100%">
                            <Stop offset="0%" stopColor="#F59E0B" />
                            <Stop offset="50%" stopColor="#FBBF24" />
                            <Stop offset="100%" stopColor="#F59E0B" />
                        </LinearGradient>
                    </Defs>

                    <G>
                        {/* Das "P" */}
                        <Path
                            d="M30 20 L30 80 Q30 85 35 85 Q40 85 40 80 L40 55"
                            stroke="url(#splashLogoGradient)"
                            strokeWidth="8"
                            strokeLinecap="round"
                            fill="none"
                        />
                        <Path
                            d="M40 20 Q70 20 70 37.5 Q70 55 40 55"
                            stroke="url(#splashLogoGradient)"
                            strokeWidth="8"
                            strokeLinecap="round"
                            fill="none"
                        />
                        <Path
                            d="M48 32 A6 6 0 1 1 48 44 A6 6 0 1 1 48 32"
                            fill="url(#splashLogoGradient)"
                        />
                    </G>
                </Svg>

                {/* Animierte Wellen - Links */}
                <Animated.View style={[styles.waveContainer, styles.waveLeft, { opacity: wave1Opacity }]}>
                    <Svg width={logoSize * 0.3} height={logoSize} viewBox="0 0 30 100">
                        <Path
                            d="M22 25 Q5 50 22 75"
                            stroke="url(#splashWaveGradient)"
                            strokeWidth="4"
                            strokeLinecap="round"
                            fill="none"
                        />
                    </Svg>
                </Animated.View>
                <Animated.View style={[styles.waveContainer, styles.waveLeftOuter, { opacity: wave2Opacity }]}>
                    <Svg width={logoSize * 0.3} height={logoSize} viewBox="0 0 30 100">
                        <Path
                            d="M25 20 Q2 50 25 80"
                            stroke="url(#splashWaveGradient)"
                            strokeWidth="3"
                            strokeLinecap="round"
                            fill="none"
                        />
                    </Svg>
                </Animated.View>

                {/* Animierte Wellen - Rechts */}
                <Animated.View style={[styles.waveContainer, styles.waveRight, { opacity: wave1Opacity }]}>
                    <Svg width={logoSize * 0.3} height={logoSize} viewBox="0 0 30 100">
                        <Path
                            d="M8 25 Q25 50 8 75"
                            stroke="url(#splashWaveGradient)"
                            strokeWidth="4"
                            strokeLinecap="round"
                            fill="none"
                        />
                    </Svg>
                </Animated.View>
                <Animated.View style={[styles.waveContainer, styles.waveRightOuter, { opacity: wave2Opacity }]}>
                    <Svg width={logoSize * 0.3} height={logoSize} viewBox="0 0 30 100">
                        <Path
                            d="M5 20 Q28 50 5 80"
                            stroke="url(#splashWaveGradient)"
                            strokeWidth="3"
                            strokeLinecap="round"
                            fill="none"
                        />
                    </Svg>
                </Animated.View>
            </Animated.View>

            {/* App Name */}
            <Animated.View style={[styles.textContainer, { opacity: textFade }]}>
                <ThemedText style={styles.logoTextPod}>Pod</ThemedText>
                <ThemedText style={styles.logoTextFetch}>Fetch</ThemedText>
            </Animated.View>
        </View>
    );
};

const styles = StyleSheet.create({
    container: {
        flex: 1,
        backgroundColor: '#0a1628',
        justifyContent: 'center',
        alignItems: 'center',
    },
    starsContainer: {
        ...StyleSheet.absoluteFillObject,
    },
    star: {
        position: 'absolute',
        backgroundColor: '#fff',
        borderRadius: 10,
    },
    logoContainer: {
        position: 'relative',
        alignItems: 'center',
        justifyContent: 'center',
    },
    waveContainer: {
        position: 'absolute',
    },
    waveLeft: {
        left: -20,
        top: 0,
    },
    waveLeftOuter: {
        left: -35,
        top: 0,
    },
    waveRight: {
        right: -20,
        top: 0,
    },
    waveRightOuter: {
        right: -35,
        top: 0,
    },
    textContainer: {
        flexDirection: 'row',
        marginTop: 20,
    },
    logoTextPod: {
        fontSize: 32,
        fontWeight: '700',
        color: '#F59E0B',
    },
    logoTextFetch: {
        fontSize: 32,
        fontWeight: '700',
        color: '#D4A574',
    },
});

export default SplashScreen;

