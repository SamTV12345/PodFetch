import {components} from "@/schema";
import {Image, Pressable, Text, View} from "react-native";
import FontAwesome from '@expo/vector-icons/FontAwesome';
import {styles} from "@/styles/styles";
import Svg, {Circle} from "react-native-svg";
import {useCallback, useEffect, useMemo, useRef} from "react";
import AntDesign from '@expo/vector-icons/AntDesign';
import {useTranslation} from "react-i18next";
import {router} from 'expo-router';
import {$api} from "@/client";
import {createAudioPlayer} from 'expo-audio';

type PodcastEpisodeSlideProps = {
    episode: components["schemas"]["PodcastEpisodeWithHistory"],
}

type ProgressCircleProps = {
    episode: components["schemas"]["PodcastEpisodeWithHistory"]
    progress?: number,
    onPlayPress: () => void,
}

type PlaybackStatus = {
    isLoaded: boolean;
    error?: string;
    isPlaying?: boolean;
    isBuffering?: boolean;
    didJustFinish?: boolean;
    positionMillis?: number;
    durationMillis?: number;
}

const ProgressCircle = ({progress = 50, episode, onPlayPress}: ProgressCircleProps) => {
    const radius = 15;
    const strokeWidth = 1;
    const circumference = 2 * Math.PI * radius;
    const progressStroke = (progress / 100) * circumference;

    return (
        <View style={{position: 'relative', height: 35, width: 35}}>
            <Svg viewBox="0 0 40 40" style={{position: 'absolute', left: 0, top: 0}}>
                <Circle
                    cx="20"
                    cy="20"
                    r={radius}
                    stroke="white"
                    strokeWidth={strokeWidth}
                    fill="none"
                    strokeDasharray={circumference}
                    strokeDashoffset={circumference - progressStroke}
                    strokeLinecap="round"
                    transform="rotate(-90 20 20)"
                />
            </Svg>
            <Pressable
                style={{
                    backgroundColor: styles.lightgray,
                    left: 5,
                    top: 5,
                    height: 25,
                    width: 25,
                    borderRadius: 50,
                    position: 'absolute',
                    justifyContent: 'center',
                    alignItems: 'center'
                }}
                onPress={onPlayPress}
            >
                <FontAwesome name="play" color="white"/>
            </Pressable>
        </View>
    );
};

export const PodcastEpisodeSlide = ({episode}: PodcastEpisodeSlideProps) => {
    const {t} = useTranslation();

    const totalProgressPercentage = useMemo(() => {
        return ((episode.podcastHistoryItem?.position || 0) /
            (episode.podcastHistoryItem?.total || 1)) * 100;
    }, [episode]);

    const remainingSeconds = useMemo(() => {
        const position = episode.podcastHistoryItem?.position;
        const total = episode.podcastHistoryItem?.total || episode.podcastEpisode.total_time;
        const isUnPlayed = position === undefined;
        const isCompletelyPlayed = position === total;

        return {
            time: Math.floor(
                (isUnPlayed || isCompletelyPlayed
                        ? episode.podcastEpisode.total_time
                        : total - (position || 0)
                ) / 60
            ),
            alreadyPlaying: position !== undefined && position !== total
        };
    }, [episode]);

    const episodeQuery = $api.useQuery("get", "/api/v1/podcasts/episode/{id}", {
        params: {
            path: {
                id: episode.podcastEpisode.episode_id
            }
        },
        enabled: false
    });

    const handlePlaybackStatus = useCallback((status: PlaybackStatus) => {
        if (!status.isLoaded) {
            if (status.error) {
                console.error(`Playback error: ${status.error}`);
            }
            return;
        }

        // Hier können Sie den Wiedergabestatus in Ihrem globalen Zustand aktualisieren
    }, []);

    const handlePlay = useCallback(async () => {
        const result = await episodeQuery.refetch();
        if (result.isSuccess) {
            const player = createAudioPlayer(episode.podcastEpisode.local_url);
            player.play();
            player.addListener('playbackStatusUpdate', handlePlaybackStatus);
        }
    }, [episode, episodeQuery]);

    return (
        <View style={{marginBottom: 10}}>
            <View style={{display: 'flex', flexDirection: 'row', marginTop: 10}}>
                <View>
                    <View style={{position: 'relative'}}>
                        <Image
                            style={{width: 50, height: 50, borderRadius: 5}}
                            source={{uri: episode.podcastEpisode.local_image_url}}
                        />
                        {totalProgressPercentage === 100 && (
                            <AntDesign
                                name="checkcircle"
                                size={24}
                                color="white"
                                style={{position: 'absolute', bottom: 1}}
                            />
                        )}
                    </View>
                </View>
                <View style={{width: '80%'}}>
                    <Pressable
                        onPress={() => {
                            router.push({
                                pathname: '/episodes/[id]',
                                params: {id: episode.podcastEpisode.id}
                            });
                        }}
                    >
                        <Text style={{
                            color: 'white',
                            marginLeft: 10
                        }} numberOfLines={2}>
                            {episode.podcastEpisode.name}
                        </Text>
                        <Text style={{
                            color: styles.whiteSubText,
                            marginLeft: 10
                        }} numberOfLines={2}>
                            {episode.podcastEpisode.description}
                        </Text>
                    </Pressable>
                    <View style={{
                        marginLeft: 10,
                        marginTop: 'auto',
                        display: 'flex',
                        flexDirection: 'row'
                    }}>
                        <ProgressCircle
                            progress={totalProgressPercentage}
                            episode={episode}
                            onPlayPress={handlePlay}
                        />
                        <Text style={{
                            color: 'white',
                            marginTop: 'auto',
                            marginBottom: 'auto'
                        }}>
                            {t(remainingSeconds.alreadyPlaying ? 'time-left' : 'time',
                                {time: remainingSeconds.time})}
                        </Text>
                    </View>
                </View>
            </View>
        </View>
    );
};