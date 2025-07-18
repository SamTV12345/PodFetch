import {Image, View} from "react-native";
import {FC} from "react";
import {components} from "@/schema";
import {ThemedText} from "@/components/ThemedText";
import {Link} from "expo-router";

export const PodcastCard: FC<{podcast: components["schemas"]["PodcastDto"]}> = ({podcast})=>{
    return <Link style={{maxWidth: 100}} href={{pathname: '/podcasts/[id]/details', params: {id: podcast.id}}}>
        <View>
        <Image    style={{width: 100, height: 100}}
                  src={podcast.image_url}/>
        <ThemedText style={{color: 'white'}} numberOfLines={2}>{podcast.name}</ThemedText>
    </View>
    </Link>
}
