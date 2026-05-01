import {Image, Pressable, Text, View, useWindowDimensions} from "react-native";
import {styles} from "@/styles/styles";
import {useStore} from "@/store/store";
import {Ionicons, MaterialCommunityIcons} from "@expo/vector-icons";
import {router} from 'expo-router';
import {useCallback, useState} from 'react';
import {useCastControls} from "@/hooks/useCastSession";

type AudioPlayerProps = {
    bottomOffset?: number;
};

export const AUDIO_PLAYER_HEIGHT = 70;


export const AudioPlayer = ({ bottomOffset = 30 }: AudioPlayerProps) => {
    const { width: screenWidth } = useWindowDimensions();
    const selectedPodcastEpisode = useStore(state => state.podcastEpisodeRecord);
    const isPlaying = useStore(state => state.isPlaying);
    const audioPercent = useStore(state => state.audioProgress);
    const audioPlayer = useStore(state => state.audioPlayer);
    const setIsPlaying = useStore(state => state.setIsPlaying);
    const castSession = useStore(state => state.castSession);
    const castStatus = useStore(state => state.castStatus);
    const { sendCommand } = useCastControls();

    const [isToggling, setIsToggling] = useState(false);

    const isCasting = !!castSession;
    const remoteState = castStatus?.state ?? castSession?.state;
    const remoteIsPlaying = remoteState === 'playing';
    const effectivePlaying = isCasting ? remoteIsPlaying : isPlaying;
    const totalSecs = selectedPodcastEpisode?.podcastEpisode.total_time ?? 0;
    const remotePercent = isCasting && totalSecs > 0
        ? ((castStatus?.position_secs ?? castSession?.position_secs ?? 0) / totalSecs) * 100
        : 0;
    const effectivePercent = isCasting ? remotePercent : audioPercent;

    const isSmallScreen = screenWidth < 375;
    const albumArtSize = isSmallScreen ? 40 : 48;
    const playButtonSize = isSmallScreen ? 32 : 36;
    const iconSize = isSmallScreen ? 18 : 20;
    const titleFontSize = isSmallScreen ? 13 : 14;
    const subtitleFontSize = isSmallScreen ? 11 : 12;

    const handleTogglePlay = useCallback((e: any) => {
        e.stopPropagation();

        if (isCasting) {
            sendCommand({ cmd: remoteIsPlaying ? 'pause' : 'resume' });
            return;
        }

        if (isToggling || !audioPlayer) return;
        setIsToggling(true);

        if (isPlaying) {
            audioPlayer.pause();
            setIsPlaying(false);
        } else {
            audioPlayer.play();
            setIsPlaying(true);
        }

        setTimeout(() => setIsToggling(false), 300);
    }, [isPlaying, audioPlayer, setIsPlaying, isToggling, isCasting, remoteIsPlaying, sendCommand]);

    const handleOpenFullscreen = () => {
        router.push('/player');
    };

    if (!selectedPodcastEpisode) {
        return null;
    }

    return (
        <Pressable
            onPress={handleOpenFullscreen}
            style={{
                position: 'absolute',
                bottom: bottomOffset,
                left: 10,
                right: 10,
                backgroundColor: styles.darkColor,
                borderRadius: 12,
                overflow: 'hidden',
                shadowColor: '#000',
                shadowOffset: { width: 0, height: 4 },
                shadowOpacity: 0.3,
                shadowRadius: 8,
                elevation: 8,
            }}
        >
            {/* Progress Bar - oben */}
            <View style={{
                height: 3,
                backgroundColor: styles.gray,
                width: '100%',
            }}>
                <View style={{
                    height: '100%',
                    width: `${effectivePercent}%`,
                    backgroundColor: styles.accentColor,
                }}/>
            </View>

            <View style={{
                flexDirection: 'row',
                alignItems: 'center',
                padding: isSmallScreen ? 8 : 10,
                paddingLeft: isSmallScreen ? 10 : 12,
                paddingRight: isSmallScreen ? 10 : 12,
            }}>
                {/* Album Art */}
                <Image
                    source={{uri: selectedPodcastEpisode.podcastEpisode.local_image_url}}
                    style={{
                        width: albumArtSize,
                        height: albumArtSize,
                        borderRadius: 6,
                    }}
                />

                {/* Text Content */}
                <View style={{
                    flex: 1,
                    marginLeft: isSmallScreen ? 10 : 12,
                    marginRight: isSmallScreen ? 10 : 12,
                    justifyContent: 'center',
                }}>
                    <Text
                        style={{
                            color: 'white',
                            fontSize: titleFontSize,
                            fontWeight: '600',
                        }}
                        numberOfLines={1}
                    >
                        {selectedPodcastEpisode.podcastEpisode.name}
                    </Text>
                    <Text
                        style={{
                            color: styles.gray,
                            fontSize: subtitleFontSize,
                            marginTop: 2,
                        }}
                        numberOfLines={1}
                    >
                        {selectedPodcastEpisode.podcast?.name || ''}
                    </Text>
                </View>

                {/* Controls */}
                <View style={{
                    flexDirection: 'row',
                    alignItems: 'center',
                    gap: isSmallScreen ? 12 : 16,
                }}>
                    {isCasting && (
                        <MaterialCommunityIcons
                            name="cast-connected"
                            size={iconSize}
                            color={styles.accentColor}
                        />
                    )}
                    {/* Play/Pause Button */}
                    <Pressable
                        onPress={handleTogglePlay}
                        style={{
                            width: playButtonSize,
                            height: playButtonSize,
                            borderRadius: playButtonSize / 2,
                            backgroundColor: 'white',
                            justifyContent: 'center',
                            alignItems: 'center',
                        }}
                    >
                        {effectivePlaying ? (
                            <Ionicons name="pause" size={iconSize} color={styles.darkColor} />
                        ) : (
                            <Ionicons name="play" size={iconSize} color={styles.darkColor} style={{marginLeft: 2}} />
                        )}
                    </Pressable>
                </View>
            </View>
        </Pressable>
    );
};

