import {Image, Pressable, View, Text, useWindowDimensions} from "react-native";
import {ThemedText} from "@/components/ThemedText";
import {FC, useMemo} from "react";
import {components} from "@/schema";
import {useStore} from "@/store/store";
import { DownloadStatusIcon } from "@/components/DownloadButton";
import {styles} from "@/styles/styles";
import { useTranslation } from "react-i18next";

export const PodcastEpisodeCard: FC<{podcastEpisode: components["schemas"]["PodcastWatchedEpisodeModelWithPodcastEpisode"]}> = ({podcastEpisode})=>{
    const { t } = useTranslation();
    const { width: screenWidth } = useWindowDimensions();

    const cardSize = Math.min(Math.max(screenWidth * 0.24, 80), 120);
    const isSmallCard = cardSize < 100;

    const progressData = useMemo(() => {
        const position = podcastEpisode.episode?.position ?? 0;
        const total = podcastEpisode.episode?.total ?? podcastEpisode.podcastEpisode.total_time ?? 0;
        const progressPercent = total > 0 ? Math.min((position / total) * 100, 100) : 0;

        const remainingSeconds = total - position;
        const remainingMinutes = Math.max(0, Math.floor(remainingSeconds / 60));

        return {
            progressPercent,
            remainingMinutes,
            hasProgress: position > 0 && progressPercent < 100,
            isCompleted: progressPercent >= 100,
        };
    }, [podcastEpisode]);

    return <Pressable style={{maxWidth: cardSize}} onPress={()=>{
        useStore.getState().setPodcastEpisodeRecord(podcastEpisode)
    }}>
        <View style={{position: 'relative'}}>
            <Image style={{width: cardSize, height: cardSize, borderRadius: 8}}
                   src={podcastEpisode.podcastEpisode.local_image_url}/>

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
                    episodeId={podcastEpisode.podcastEpisode.episode_id}
                    size={isSmallCard ? 12 : 14}
                />
            </View>
        </View>
        <ThemedText style={{color: 'white', fontSize: isSmallCard ? 12 : 14}} numberOfLines={2}>{podcastEpisode.podcastEpisode.name}</ThemedText>

        {/* Verbleibende Zeit anzeigen */}
        {progressData.hasProgress && (
            <Text style={{
                color: styles.gray,
                fontSize: isSmallCard ? 10 : 11,
                marginTop: 2,
            }}>
                {t('minutes-remaining-short', { minutes: progressData.remainingMinutes })}
            </Text>
        )}
    </Pressable>
}
