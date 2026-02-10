import {components} from "@/schema";
import {Image, Pressable, Text, View} from "react-native";
import FontAwesome from '@expo/vector-icons/FontAwesome';
import {styles} from "@/styles/styles";
import Svg, {Circle} from "react-native-svg";
import {useCallback, useMemo} from "react";
import AntDesign from '@expo/vector-icons/AntDesign';
import {useTranslation} from "react-i18next";
import {router} from 'expo-router';
import {useStore} from "@/store/store";
import { DownloadButton, DownloadStatusIcon } from "@/components/DownloadButton";

type PodcastEpisodeSlideProps = {
    episode: components["schemas"]["PodcastEpisodeWithHistory"],
    podcast?: components["schemas"]["PodcastDto"],
}

type ProgressCircleProps = {
    progress?: number,
    onPlayPress: () => void,
    isPlaying?: boolean,
    isCurrentEpisode?: boolean,
}

const ProgressCircle = ({progress = 50, onPlayPress, isPlaying = false, isCurrentEpisode = false}: ProgressCircleProps) => {
    const radius = 15;
    const strokeWidth = 1;
    const circumference = 2 * Math.PI * radius;
    const progressStroke = (progress / 100) * circumference;

    // Zeige Pause-Icon wenn diese Episode gerade spielt
    const showPauseIcon = isCurrentEpisode && isPlaying;

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
                <FontAwesome name={showPauseIcon ? "pause" : "play"} color="white"/>
            </Pressable>
        </View>
    );
};

export const PodcastEpisodeSlide = ({episode, podcast}: PodcastEpisodeSlideProps) => {
    const {t} = useTranslation();
    const playEpisode = useStore(state => state.playEpisode);
    const currentEpisode = useStore(state => state.podcastEpisodeRecord);
    const isPlaying = useStore(state => state.isPlaying);
    const audioPlayer = useStore(state => state.audioPlayer);
    const setIsPlaying = useStore(state => state.setIsPlaying);

    const isCurrentEpisode = currentEpisode?.podcastEpisode.episode_id === episode.podcastEpisode.episode_id;

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

    const handlePlay = useCallback(() => {
        console.log("Playing episode:", episode.podcastEpisode.name);

        // Wenn diese Episode bereits geladen ist, nur Play/Pause toggeln
        if (isCurrentEpisode && audioPlayer) {
            if (isPlaying) {
                audioPlayer.pause();
                setIsPlaying(false);
            } else {
                audioPlayer.play();
                setIsPlaying(true);
            }
        } else {
            // Neue Episode starten
            playEpisode(episode.podcastEpisode);
        }
    }, [episode, playEpisode, isCurrentEpisode, audioPlayer, isPlaying, setIsPlaying]);

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
                                name="check-circle"
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
                        flexDirection: 'row',
                        alignItems: 'center',
                        gap: 8,
                    }}>
                        <ProgressCircle
                            progress={totalProgressPercentage}
                            onPlayPress={handlePlay}
                            isPlaying={isPlaying}
                            isCurrentEpisode={isCurrentEpisode}
                        />
                        <Text style={{
                            color: 'white',
                            marginTop: 'auto',
                            marginBottom: 'auto'
                        }}>
                            {t(remainingSeconds.alreadyPlaying ? 'time-left' : 'time',
                                {time: remainingSeconds.time})}
                        </Text>
                        {/* Download Status Icon (nur anzeigen wenn heruntergeladen) */}
                        <DownloadStatusIcon
                            episodeId={episode.podcastEpisode.episode_id}
                            size={16}
                        />
                        {/* Download Button (nur anzeigen wenn podcast vorhanden) */}
                        {podcast && (
                            <DownloadButton
                                episode={episode.podcastEpisode}
                                podcast={podcast}
                                size={20}
                                showProgress={true}
                            />
                        )}
                    </View>
                </View>
            </View>
        </View>
    );
};