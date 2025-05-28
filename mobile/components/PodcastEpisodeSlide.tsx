import {components} from "@/schema";
import {Image, Pressable, Text, View} from "react-native";
import FontAwesome from '@expo/vector-icons/FontAwesome';
import {styles} from "@/styles/styles";
import Svg, {Circle} from "react-native-svg";
import {useMemo} from "react";
import AntDesign from '@expo/vector-icons/AntDesign';
import {useTranslation} from "react-i18next";
import { router } from 'expo-router';

type PodcastEpisodeSlideProps = {
    episode: components["schemas"]["PodcastEpisodeWithHistory"],
}

const ProgressCircle = ({ progress = 50 }) => {
    const radius = 15; // Radius of the circle
    const strokeWidth = 1;
    const circumference = 2 * Math.PI * radius;
    const progressStroke = (progress / 100) * circumference;

    return (
        <View style={{position: 'relative', height: 35, width: 35}} >
            <Svg viewBox="0 0 40 40" style={{position: 'absolute', left: 0, top: 0}}>
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
            <View style={{backgroundColor: styles.lightgray, left: 5, top: 5, height: 25, width: 25, borderRadius: 50, position: 'absolute'}}>
                <FontAwesome name="play" color="white" style={{margin: 'auto'}}/>
            </View>
        </View>
    );
};

export const PodcastEpisodeSlide = ({ episode }: PodcastEpisodeSlideProps) => {
    const totalProgressPercentage = useMemo(()=>{
        return (episode.podcastHistoryItem?.position||0)/(episode.podcastHistoryItem?.total||1)*100
    }, [episode])
    const {t} = useTranslation()


    const remainingSeconds = useMemo(()=>{
        const isUnPlayed = episode.podcastHistoryItem?.position === undefined
        const isCompletelyPlayed = episode.podcastHistoryItem?.position === episode.podcastHistoryItem?.total
        if(isUnPlayed || isCompletelyPlayed){
            return {
                time: Math.floor(episode.podcastEpisode.total_time/60),
                alreadyPlaying: false
            }
        }

        return {
            time: Math.floor(((episode.podcastHistoryItem?.total|| episode.podcastEpisode.total_time) - (episode.podcastHistoryItem?.position||0))/60),
            alreadyPlaying: episode.podcastHistoryItem?.position != undefined && episode.podcastHistoryItem?.position != episode.podcastHistoryItem?.total
        }
    } ,[episode])

    return <View style={{marginBottom: 10}}>
        <View  style={{display: 'flex', flexDirection: 'row', marginTop: 10}}>
            <View>
                <View  style={{position: 'relative'}}>
                    <Image style={{ width: 50, height: 50, borderRadius: 5 }}
                           src={episode.podcastEpisode.local_image_url} />
                    {totalProgressPercentage == 100 && <AntDesign name="checkcircle" size={24} color="white" style={{position: 'absolute', bottom: 1}} />}
                </View>
            </View>
            <View style={{width: '80%'}}>
                <Pressable onPress={()=>{
                    router.push({
                        pathname: '/episodes/[id]',
                        params: {
                            id: episode.podcastEpisode.id,
                        }
                    })
                }}>
                    <Text style={{ color: 'white', wordWrap: "break", marginLeft: 10 }} numberOfLines={2}>{episode.podcastEpisode.name}</Text>
                    <Text style={{ color: styles.whiteSubText, wordWrap: "break", marginLeft: 10 }} numberOfLines={2}>{episode.podcastEpisode.description}</Text>
                </Pressable>
                <View style={{marginLeft: 10, marginTop: 'auto', display: 'flex', flexDirection: 'row'}}>
                    <ProgressCircle progress={totalProgressPercentage} />
                    {remainingSeconds.alreadyPlaying ? <Text style={{color: 'white', marginTop: 'auto', marginBottom: 'auto'}}>{t('time-left', {time: remainingSeconds.time})}</Text>:
                        <Text style={{color: 'white', marginTop: 'auto', marginBottom: 'auto'}}>{t('time', {time: remainingSeconds.time})}</Text>}
                </View>
            </View>
        </View>
    </View>
}
