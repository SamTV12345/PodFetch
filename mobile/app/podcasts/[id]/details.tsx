import {Image, ScrollView, Share, Text, View} from "react-native";
import {SafeAreaView} from "react-native-safe-area-context";
import {Link} from "expo-router";
import MaterialIcons from "@expo/vector-icons/MaterialIcons";
import Heading2 from "@/components/text/Heading2";
import AntDesign from '@expo/vector-icons/AntDesign';
import Entypo from '@expo/vector-icons/Entypo';
import {PodcastEpisodeSlide} from "@/components/PodcastEpisodeSlide";
import { router } from 'expo-router';
import {useTranslation} from "react-i18next";
import {useDetailsPodcast} from "@/hooks/useDetailsPodcast";
import {AudioPlayer} from "@/components/AudioPlayer";

export default function () {
    const {podcastDetailedData, dataEpisodes, updateFavored} = useDetailsPodcast()
    const {t} = useTranslation()

    return <SafeAreaView style={{flex: 1}}>
        <ScrollView overScrollMode="never">
                <MaterialIcons size={40} color="white" name="chevron-left" style={{
                    position: 'absolute',
                    top: 20,
                    left: 20,
                }} onPress={()=>{
                    router.push('/(tabs)')
                }} />
        {
            !podcastDetailedData.isLoading && podcastDetailedData.data? <>
                <Image src={podcastDetailedData.data.image_url}   style={{
                    width: 200,
                    height: 200,
                    borderRadius: 20,
                    marginLeft: 'auto',
                    marginRight: 'auto',
                    marginTop: 50,
                    shadowColor: '#000',
                    shadowOffset: { width: 0, height: 2 },
                    shadowOpacity: 0.8,
                    shadowRadius: 10,
                }}
                />
                <Heading2 styles={{marginRight: 'auto', marginLeft: 'auto', width: '95%', marginTop: 10, paddingBottom: 0}}>{podcastDetailedData.data.name}</Heading2>
                {podcastDetailedData.data.tags.map(t=><Text>{t.name}</Text>)}
                <View style={{marginLeft: 30, display: 'flex', flexDirection: 'row', gap: 10}}>
                    {
                        podcastDetailedData.data.favorites ? <AntDesign name="heart" size={24} color="red" onPress={()=>{
                            updateFavored.mutate({})
                        }} /> : <AntDesign name="heart" onPress={()=>{
                            updateFavored.mutate({})
                        }}/>
                    }
                    <Link href={{pathname: "/podcasts/[id]/info", params: {
                                id: podcastDetailedData.data.id
                            }}}>
                        <AntDesign name="info-circle" size={24} color="white" />
                    </Link>
                    <Entypo name="share-alternative" size={24} color="white" onPress={()=>{
                       Share.share({
                           message: t('share-podcast', {name: podcastDetailedData.data.name, url: "tbd"}),

                       })
                    }} />
                </View>
                <View style={{margin: 20}}>
                    {
                        !dataEpisodes.isLoading && dataEpisodes.data!.map(d=>{
                            return <PodcastEpisodeSlide episode={d} key={d.podcastEpisode.id}/>
                        })
                    }
                </View>
            </>: <>
            </>
        }
        </ScrollView>
        <AudioPlayer bottomOffset={20} />
    </SafeAreaView>
}
