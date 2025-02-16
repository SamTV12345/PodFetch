import {Image, Pressable} from "react-native";
import {ThemedText} from "@/components/ThemedText";
import {FC} from "react";
import {components} from "@/schema";
import {useStore} from "@/store/store";

export const PodcastEpisodeCard: FC<{podcastEpisode: components["schemas"]["PodcastWatchedEpisodeModelWithPodcastEpisode"]}> = ({podcastEpisode})=>{
    return <Pressable style={{maxWidth: 100}} onPress={()=>{
        useStore.getState().setPodcastEpisodeRecord(podcastEpisode)
    }}>
        <Image    style={{width: 100, height: 100}}
                  src={podcastEpisode.podcastEpisode.local_image_url}/>
        <ThemedText style={{color: 'white', wordWrap: "break"}} numberOfLines={2}>{podcastEpisode.podcastEpisode.name}</ThemedText>
    </Pressable>
}
