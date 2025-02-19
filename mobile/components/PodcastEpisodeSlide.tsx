import {components} from "@/schema";
import {Image, Text, View} from "react-native";
import FontAwesome from '@expo/vector-icons/FontAwesome';
import {styles} from "@/styles/styles";
import Svg, {Circle} from "react-native-svg";

type PodcastEpisodeSlideProps = {
    episode: components["schemas"]["PodcastEpisodeWithHistory"],
}

const ProgressCircle = ({ progress = 50 }) => {
    const radius = 15; // Radius of the circle
    const strokeWidth = 1;
    const circumference = 2 * Math.PI * radius;
    const progressStroke = (progress / 100) * circumference;

    return (
        <View>
            <Svg height={35} width={35} viewBox="0 0 40 40">
                {/* Progress Circle */}
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
            <View style={{backgroundColor: styles.lightgray, height: 25, width: 25, borderRadius: 50, position: 'absolute'}}>
                <FontAwesome name="play" color="white" style={{margin: 'auto'}}/>
            </View>
        </View>
    );
};

export const PodcastEpisodeSlide = ({ episode }: PodcastEpisodeSlideProps) => {

    return <View>
        <View  style={{display: 'flex', flexDirection: 'row', marginTop: 10, marginBottom: 10}}>
            <Image style={{ width: 50, height: 50, borderRadius: 5 }}
                             src={episode.podcastEpisode.local_image_url} />
            <Text style={{ color: 'white', wordWrap: "break", marginLeft: 10 }} numberOfLines={2}>{episode.podcastEpisode.name}</Text>
        </View>
        <View>
            <ProgressCircle progress={(episode.podcastHistoryItem?.position||0)/(episode.podcastHistoryItem?.total||1)} />
        </View>
    </View>
}
