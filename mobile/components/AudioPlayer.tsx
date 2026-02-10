import {Image, Pressable, Text, View} from "react-native";
import {styles} from "@/styles/styles";
import {useStore} from "@/store/store";
import {Ionicons} from "@expo/vector-icons";
import {router} from 'expo-router';
import {useCallback, useState} from 'react';

type AudioPlayerProps = {
    /** Abstand vom unteren Rand - default 80 für Tabs, 20 für andere Seiten */
    bottomOffset?: number;
};

/**
 * Mini Audio Player Bar - zeigt nur die UI an.
 * Der eigentliche Audio-Player wird vom AudioProvider verwaltet.
 */
export const AudioPlayer = ({ bottomOffset = 80 }: AudioPlayerProps) => {
    const selectedPodcastEpisode = useStore(state => state.podcastEpisodeRecord);
    const isPlaying = useStore(state => state.isPlaying);
    const audioPercent = useStore(state => state.audioProgress);
    const audioPlayer = useStore(state => state.audioPlayer);
    const setIsPlaying = useStore(state => state.setIsPlaying);

    const [isToggling, setIsToggling] = useState(false);

    const handleTogglePlay = useCallback((e: any) => {
        e.stopPropagation();

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
    }, [isPlaying, audioPlayer, setIsPlaying, isToggling]);

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
                    width: `${audioPercent}%`,
                    backgroundColor: styles.accentColor,
                }}/>
            </View>

            {/* Content */}
            <View style={{
                flexDirection: 'row',
                alignItems: 'center',
                padding: 10,
                paddingLeft: 12,
                paddingRight: 12,
            }}>
                {/* Album Art */}
                <Image
                    source={{uri: selectedPodcastEpisode.podcastEpisode.local_image_url}}
                    style={{
                        width: 48,
                        height: 48,
                        borderRadius: 6,
                    }}
                />

                {/* Text Content */}
                <View style={{
                    flex: 1,
                    marginLeft: 12,
                    marginRight: 12,
                    justifyContent: 'center',
                }}>
                    <Text
                        style={{
                            color: 'white',
                            fontSize: 14,
                            fontWeight: '600',
                        }}
                        numberOfLines={1}
                    >
                        {selectedPodcastEpisode.podcastEpisode.name}
                    </Text>
                    <Text
                        style={{
                            color: styles.gray,
                            fontSize: 12,
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
                    gap: 16,
                }}>
                    {/* Play/Pause Button */}
                    <Pressable
                        onPress={handleTogglePlay}
                        style={{
                            width: 36,
                            height: 36,
                            borderRadius: 18,
                            backgroundColor: 'white',
                            justifyContent: 'center',
                            alignItems: 'center',
                        }}
                    >
                        {isPlaying ? (
                            <Ionicons name="pause" size={20} color={styles.darkColor} />
                        ) : (
                            <Ionicons name="play" size={20} color={styles.darkColor} style={{marginLeft: 2}} />
                        )}
                    </Pressable>
                </View>
            </View>
        </Pressable>
    );
};

