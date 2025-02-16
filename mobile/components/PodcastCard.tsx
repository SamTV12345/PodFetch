import {Image, Text, View} from "react-native";
import {FC} from "react";
import {components} from "@/schema";
import {ThemedText} from "@/components/ThemedText";

export const PodcastCard: FC<{podcast: components["schemas"]["PodcastDto"]}> = ({podcast})=>{
    return <View style={{maxWidth: 100}}>
        <Image    style={{width: 100, height: 100}}
                  src={podcast.image_url}/>
        <ThemedText style={{color: 'white', wordWrap: "break"}} numberOfLines={2}>{podcast.name}</ThemedText>
    </View>
}
