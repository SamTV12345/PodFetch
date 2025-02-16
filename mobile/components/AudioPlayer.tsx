import {Image, ScrollView, Text, View} from "react-native";
import {styles} from "@/styles/styles";
import {useStore} from "@/store/store";
import {IconSymbol} from "@/components/ui/IconSymbol";
import {AntDesign} from "@expo/vector-icons";
import MaterialIcons from "@expo/vector-icons/MaterialIcons";

export const AudioPlayer = ()=>{
    const selectedPodcastEpisode = useStore(state=>state.podcastEpisodeRecord)
    const changeAudioState = useStore(state=>state.togglePlaying)
    const playingAudio = useStore(state=>state.isPlaying)
    const audioPercent = useStore(state=>state.audioProgress)


    return selectedPodcastEpisode?<View style={{bottom: 10, marginLeft: 18, marginRight: 18, backgroundColor: styles.darkColor, display: 'flex', flexDirection: 'row', position: 'relative'}}>
        <View style={{width: `${audioPercent}%`, backgroundColor: 'white', height: 2, position: 'absolute', borderTopLeftRadius: 20, borderTopRightRadius: 20}}></View>
        <View style={{width: '100%', backgroundColor: styles.gray, height: 2, position: 'absolute', borderTopLeftRadius: 20, borderTopRightRadius: 20}}></View>
        <Image src={selectedPodcastEpisode?.podcastEpisode.local_image_url} style={{width: 40, height: 40,alignSelf: 'center', marginLeft: 10, marginTop: 10}}/>
        <View style={{flexDirection: 'column', marginTop: 10, width: '90%'}}>
            <ScrollView horizontal style={{ marginLeft: 10, marginRight: 80}} overScrollMode="never">
                <Text style={{color: 'white', fontSize: 15 }}>{selectedPodcastEpisode?.podcastEpisode.name}</Text>
            </ScrollView>
            <Text style={{color: styles.gray, fontSize: 13, marginRight: 85, marginLeft: 10, }} numberOfLines={1}>{selectedPodcastEpisode?.podcast.name}</Text>
        </View>
        <View style={{position: 'absolute', right: 10, alignSelf: 'center', paddingTop: 10, display: 'flex', flexDirection: 'row', gap: 3}}>
            <MaterialIcons name="ios-share" color="white" size={20}/>
            {playingAudio ? <AntDesign name="pause" size={20} color="white" onPress={()=>changeAudioState()}/>:  <AntDesign size={20} name="caretright" color="white" onPress={()=>changeAudioState()}/>}
        </View>
    </View>: <View></View>

}
