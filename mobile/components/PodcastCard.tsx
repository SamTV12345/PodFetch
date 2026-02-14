import {Image, View, useWindowDimensions} from "react-native";
import {FC} from "react";
import {components} from "@/schema";
import {ThemedText} from "@/components/ThemedText";
import {Link} from "expo-router";

export const PodcastCard: FC<{podcast: components["schemas"]["PodcastDto"]}> = ({podcast})=>{
    const { width: screenWidth } = useWindowDimensions();

    const cardSize = Math.min(Math.max(screenWidth * 0.24, 80), 120);

    return <Link style={{maxWidth: cardSize}} href={{pathname: '/podcasts/[id]/details', params: {id: podcast.id}}}>
        <View>
        <Image style={{width: cardSize, height: cardSize, borderRadius: 8}}
                  src={podcast.image_url}/>
        <ThemedText style={{color: 'white', fontSize: cardSize < 100 ? 12 : 14}} numberOfLines={2}>{podcast.name}</ThemedText>
    </View>
    </Link>
}
