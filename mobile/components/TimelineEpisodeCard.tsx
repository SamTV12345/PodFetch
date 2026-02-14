import {Image, Pressable, View, Text, useWindowDimensions} from "react-native";
import {ThemedText} from "@/components/ThemedText";
import {FC, useMemo} from "react";
import {components} from "@/schema";
import {useStore} from "@/store/store";
import {DownloadStatusIcon} from "@/components/DownloadButton";
import {styles} from "@/styles/styles";

type TimelineEpisodeCardProps = {
    episode: components["schemas"]["TimeLinePodcastEpisode"];
};

export const TimelineEpisodeCard: FC<TimelineEpisodeCardProps> = ({episode}) => {
    const { width: screenWidth } = useWindowDimensions();

    const cardSize = Math.min(Math.max(screenWidth * 0.24, 80), 120);
    const isSmallCard = cardSize < 100;

    const progressData = useMemo(() => {
        const position = episode.history?.position ?? 0;
        const total = episode.history?.total ?? episode.podcast_episode.total_time ?? 0;
        const progressPercent = total > 0 ? Math.min((position / total) * 100, 100) : 0;

        const remainingSeconds = total - position;
        const remainingMinutes = Math.max(0, Math.floor(remainingSeconds / 60));

        const totalMinutes = Math.floor((episode.podcast_episode.total_time ?? 0) / 60);

        return {
            progressPercent,
            remainingMinutes,
            totalMinutes,
            hasProgress: position > 0 && progressPercent < 100,
            isCompleted: progressPercent >= 100,
            isNew: position === 0,
        };
    }, [episode]);

    const handlePress = () => {
        const record: components["schemas"]["PodcastWatchedEpisodeModelWithPodcastEpisode"] = {
            podcastEpisode: episode.podcast_episode,
            podcast: episode.podcast,
            episode: episode.history || {
                podcast: episode.podcast?.rssfeed || '',
                episode: episode.podcast_episode.url,
                timestamp: new Date().toISOString(),
                guid: episode.podcast_episode.guid,
                action: 'play',
                started: 0,
                position: 0,
                total: episode.podcast_episode.total_time,
                device: 'mobile',
            },
        };
        useStore.getState().setPodcastEpisodeRecord(record);
    };

    return (
        <Pressable style={{maxWidth: cardSize}} onPress={handlePress}>
            <View style={{position: 'relative'}}>
                <Image
                    style={{width: cardSize, height: cardSize, borderRadius: 8}}
                    source={{uri: episode.podcast_episode.local_image_url || episode.podcast?.image_url}}
                />

                {/* "Neu" Badge für ungehörte Episoden */}
                {progressData.isNew && !progressData.isCompleted && (
                    <View style={{
                        position: 'absolute',
                        top: 4,
                        left: 4,
                        backgroundColor: styles.accentColor,
                        paddingHorizontal: isSmallCard ? 4 : 6,
                        paddingVertical: 2,
                        borderRadius: 4,
                    }}>
                        <Text style={{color: '#fff', fontSize: isSmallCard ? 8 : 9, fontWeight: '700'}}>NEU</Text>
                    </View>
                )}

                {/* Fortschrittsbalken unten am Bild */}
                {progressData.hasProgress && (
                    <View style={{
                        position: 'absolute',
                        bottom: 0,
                        left: 0,
                        right: 0,
                        height: isSmallCard ? 3 : 4,
                        backgroundColor: 'rgba(0,0,0,0.5)',
                        borderBottomLeftRadius: 8,
                        borderBottomRightRadius: 8,
                        overflow: 'hidden',
                    }}>
                        <View style={{
                            height: '100%',
                            width: `${progressData.progressPercent}%`,
                            backgroundColor: styles.accentColor,
                        }}/>
                    </View>
                )}

                {/* Download-Indikator in der Ecke */}
                <View style={{position: 'absolute', bottom: progressData.hasProgress ? (isSmallCard ? 6 : 8) : 4, right: 4}}>
                    <DownloadStatusIcon
                        episodeId={episode.podcast_episode.episode_id}
                        size={isSmallCard ? 12 : 14}
                    />
                </View>
            </View>
            <ThemedText style={{color: 'white', fontSize: isSmallCard ? 12 : 14}} numberOfLines={2}>
                {episode.podcast_episode.name}
            </ThemedText>

            {/* Podcast-Name */}
            <Text style={{
                color: styles.gray,
                fontSize: isSmallCard ? 10 : 11,
                marginTop: 2,
            }} numberOfLines={1}>
                {episode.podcast?.name}
            </Text>

            {/* Zeit-Anzeige */}
            {progressData.hasProgress ? (
                <Text style={{
                    color: styles.accentColor,
                    fontSize: isSmallCard ? 9 : 10,
                    marginTop: 2,
                }}>
                    {progressData.remainingMinutes} Min übrig
                </Text>
            ) : !progressData.isCompleted && progressData.totalMinutes > 0 && (
                <Text style={{
                    color: styles.gray,
                    fontSize: isSmallCard ? 9 : 10,
                    marginTop: 2,
                }}>
                    {progressData.totalMinutes} Min
                </Text>
            )}
        </Pressable>
    );
};

