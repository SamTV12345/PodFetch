import {Image, ScrollView, Text, View} from "react-native";
import {SafeAreaView} from "react-native-safe-area-context";
import {Link, useLocalSearchParams} from "expo-router";
import {$api} from "@/client";
import MaterialIcons from "@expo/vector-icons/MaterialIcons";
import Heading2 from "@/components/text/Heading2";
import AntDesign from '@expo/vector-icons/AntDesign';
import Entypo from '@expo/vector-icons/Entypo';
import {PodcastEpisodeSlide} from "@/components/PodcastEpisodeSlide";
import { router } from 'expo-router';

export default function () {
    const { id } = useLocalSearchParams();
    const {data, isLoading} = $api.useQuery('get', '/api/v1/podcasts/{id}', {
        params: {
            path: {
                id: id as string
            }
        }
    })
    const dataEpisodes = $api.useQuery('get', '/api/v1/podcasts/{id}/episodes', {
        params: {
            path: {
                id: id as string
            }
        }
    })

    return <SafeAreaView>
        <ScrollView overScrollMode="never">
                <MaterialIcons size={40} color="white" name="chevron-left" style={{
                    position: 'absolute',
                    top: 20,
                    left: 20,
                }} onPress={()=>{
                    router.push('/(tabs)')
                }} />
        {
            !isLoading && data? <>
                <Image src={data.image_url}   style={{
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
                <Heading2 styles={{marginRight: 'auto', marginLeft: 'auto', width: '95%', marginTop: 10, paddingBottom: 0}}>{data.name}</Heading2>
                {data.tags.map(t=><Text>{t.name}</Text>)}
                <View style={{marginLeft: 30, display: 'flex', flexDirection: 'row', gap: 10}}>
                    <AntDesign name="hearto" size={24} color="white" />
                    <Link href={{pathname: "/podcasts/[id]/info", params: {
                                id: data.id
                            }}}>
                        <AntDesign name="infocirlce" size={24} color="white" />
                    </Link>
                    <Entypo name="share-alternative" size={24} color="white" />
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
    </SafeAreaView>
}
