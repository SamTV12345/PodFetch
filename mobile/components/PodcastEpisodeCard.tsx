import {Image, Pressable, View} from "react-native";
import {ThemedText} from "@/components/ThemedText";
import {FC} from "react";
import {components} from "@/schema";
import {useStore} from "@/store/store";
import { DownloadStatusIcon } from "@/components/DownloadButton";

export const PodcastEpisodeCard: FC<{podcastEpisode: components["schemas"]["PodcastWatchedEpisodeModelWithPodcastEpisode"]}> = ({podcastEpisode})=>{
    return <Pressable style={{maxWidth: 100}} onPress={()=>{
        useStore.getState().setPodcastEpisodeRecord(podcastEpisode)
    }}>
        <View style={{position: 'relative'}}>
            <Image style={{width: 100, height: 100, borderRadius: 8}}
                   src={podcastEpisode.podcastEpisode.local_image_url}/>
            {/* Download-Indikator in der Ecke */}
            <View style={{position: 'absolute', bottom: 4, right: 4}}>
                <DownloadStatusIcon
                    episodeId={podcastEpisode.podcastEpisode.episode_id}
                    size={14}
                />
            </View>
        </View>
        <ThemedText style={{color: 'white'}} numberOfLines={2}>{podcastEpisode.podcastEpisode.name}</ThemedText>
    </Pressable>
}
