import * as React from "react";
import Svg, { Path, Defs, LinearGradient, Stop, G } from "react-native-svg";
import { View } from "react-native";

type PodFetchLogoProps = {
    size?: number;
    showText?: boolean;
};

/**
 * PodFetch Logo - Das "P" mit Radiowellen
 * Inspiriert vom Podcast/Audio-Symbol
 */
export const PodFetchLogo: React.FC<PodFetchLogoProps> = ({ size = 100, showText = false }) => {
    const scale = size / 100;

    return (
        <View style={{ alignItems: 'center' }}>
            <Svg width={size} height={size} viewBox="0 0 100 100">
                <Defs>
                    {/* Gradient für das P - von Lila zu Orange/Gold */}
                    <LinearGradient id="logoGradient" x1="0%" y1="0%" x2="100%" y2="100%">
                        <Stop offset="0%" stopColor="#8B5CF6" />
                        <Stop offset="30%" stopColor="#A855F7" />
                        <Stop offset="60%" stopColor="#D946EF" />
                        <Stop offset="100%" stopColor="#F59E0B" />
                    </LinearGradient>
                    {/* Gradient für die Wellen */}
                    <LinearGradient id="waveGradient" x1="0%" y1="0%" x2="0%" y2="100%">
                        <Stop offset="0%" stopColor="#F59E0B" />
                        <Stop offset="50%" stopColor="#FBBF24" />
                        <Stop offset="100%" stopColor="#F59E0B" />
                    </LinearGradient>
                </Defs>

                <G>
                    {/* Das "P" - stilisiert wie ein Podcast-Symbol */}
                    {/* Vertikaler Strich des P */}
                    <Path
                        d="M30 20 L30 80 Q30 85 35 85 Q40 85 40 80 L40 55"
                        stroke="url(#logoGradient)"
                        strokeWidth="8"
                        strokeLinecap="round"
                        fill="none"
                    />

                    {/* Bogen des P (wie ein Kopfhörer/Ohr) */}
                    <Path
                        d="M40 20 Q70 20 70 37.5 Q70 55 40 55"
                        stroke="url(#logoGradient)"
                        strokeWidth="8"
                        strokeLinecap="round"
                        fill="none"
                    />

                    {/* Innerer Punkt (wie bei einem Mikrofon) */}
                    <Path
                        d="M48 32 A6 6 0 1 1 48 44 A6 6 0 1 1 48 32"
                        fill="url(#logoGradient)"
                    />

                    {/* Linke Radiowellen */}
                    <Path
                        d="M18 30 Q8 50 18 70"
                        stroke="url(#waveGradient)"
                        strokeWidth="4"
                        strokeLinecap="round"
                        fill="none"
                        opacity="0.9"
                    />
                    <Path
                        d="M10 25 Q-5 50 10 75"
                        stroke="url(#waveGradient)"
                        strokeWidth="3"
                        strokeLinecap="round"
                        fill="none"
                        opacity="0.6"
                    />

                    {/* Rechte Radiowellen */}
                    <Path
                        d="M78 30 Q88 50 78 70"
                        stroke="url(#waveGradient)"
                        strokeWidth="4"
                        strokeLinecap="round"
                        fill="none"
                        opacity="0.9"
                    />
                    <Path
                        d="M86 25 Q101 50 86 75"
                        stroke="url(#waveGradient)"
                        strokeWidth="3"
                        strokeLinecap="round"
                        fill="none"
                        opacity="0.6"
                    />
                </G>
            </Svg>

            {showText && (
                <View style={{ flexDirection: 'row', marginTop: size * 0.1 }}>
                    <Svg width={size * 1.5} height={size * 0.3} viewBox="0 0 150 30">
                        <Defs>
                            <LinearGradient id="textGradient" x1="0%" y1="0%" x2="100%" y2="0%">
                                <Stop offset="0%" stopColor="#F59E0B" />
                                <Stop offset="100%" stopColor="#D4A574" />
                            </LinearGradient>
                        </Defs>
                        {/* "Pod" in Gold */}
                        <Path
                            d="M5 5 L5 25 M5 5 L15 5 Q22 5 22 12 Q22 19 15 19 L5 19"
                            stroke="url(#textGradient)"
                            strokeWidth="3"
                            strokeLinecap="round"
                            fill="none"
                        />
                        <Path
                            d="M28 19 Q28 5 38 5 Q48 5 48 19 Q48 25 38 25 Q28 25 28 19 Z"
                            stroke="url(#textGradient)"
                            strokeWidth="3"
                            fill="none"
                        />
                        <Path
                            d="M55 25 L55 12 Q55 5 65 5 Q72 5 72 12 L72 25 M72 5 L72 5"
                            stroke="url(#textGradient)"
                            strokeWidth="3"
                            strokeLinecap="round"
                            fill="none"
                        />
                        {/* "Fetch" in hellerem Gold/Beige */}
                        <Path
                            d="M82 5 L82 25 M82 5 L95 5 M82 15 L92 15"
                            stroke="#D4A574"
                            strokeWidth="3"
                            strokeLinecap="round"
                            fill="none"
                        />
                        <Path
                            d="M100 12 Q100 5 110 5 Q118 5 118 8 M100 19 Q100 25 110 25 Q118 25 118 22"
                            stroke="#D4A574"
                            strokeWidth="3"
                            strokeLinecap="round"
                            fill="none"
                        />
                        <Path
                            d="M125 5 L125 25 M118 5 L132 5"
                            stroke="#D4A574"
                            strokeWidth="3"
                            strokeLinecap="round"
                            fill="none"
                        />
                        <Path
                            d="M138 5 L138 25 M138 5 L138 5 M138 15 L148 5 M138 15 L148 25"
                            stroke="#D4A574"
                            strokeWidth="3"
                            strokeLinecap="round"
                            fill="none"
                        />
                    </Svg>
                </View>
            )}
        </View>
    );
};

export default PodFetchLogo;

