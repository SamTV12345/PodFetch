import { ThemedView } from '@/components/ThemedView';
import {SafeAreaView} from "react-native-safe-area-context";
import {styles} from "@/styles/styles";
import Heading1 from "@/components/text/Heading1";
import {$api} from "@/client";
import {useTranslation} from "react-i18next";
import {ScrollView, View} from "react-native";
import Heading2 from "@/components/text/Heading2";
import {LoadingSkeleton} from "@/components/ui/LoadingSkeleton";
import {PodcastCard} from "@/components/PodcastCard";
import {PodcastEpisodeCard} from "@/components/PodcastEpisodeCard";

const HomeScreen = ()=> {
  const {data, isError, isLoading, error} = $api.useQuery('get', '/api/v1/podcasts')
  const lastWatchedData = $api.useQuery('get', '/api/v1/podcasts/episode/lastwatched')
    const {t} = useTranslation()

  return (
    <SafeAreaView>
      <ThemedView style={{
        backgroundColor: styles.lightDarkColor,
          paddingTop: 20,
          paddingLeft: 10,
          paddingRight: 10
      }}>
          <Heading1>{t('home')}</Heading1>

          <Heading2>{t('last-listened')}</Heading2>
          <ScrollView horizontal={true} style={{paddingBottom: 20, display: 'flex', gap: 10}} overScrollMode="never">
              {lastWatchedData.isLoading &&
                  <>
                  <LoadingSkeleton/>
              <LoadingSkeleton/>
              <LoadingSkeleton/>
              <LoadingSkeleton/>
              <LoadingSkeleton/>
              <LoadingSkeleton/>
              </>
              }
              <View style={{display: 'flex', gap: 10, flexDirection: 'row'}}>
              {
                  lastWatchedData.data && lastWatchedData.data.map(d=>{
                      return <PodcastEpisodeCard podcastEpisode={d} key={d.id} />
                  })
              }
              </View>
          </ScrollView>


          <Heading2 more onMore={()=>{}}>{t('your-podcasts')}</Heading2>
          <ScrollView horizontal={true} style={{paddingBottom: 20}} overScrollMode="never">
              {isLoading &&<><LoadingSkeleton/>
                  <LoadingSkeleton/>
                  <LoadingSkeleton/>
                  <LoadingSkeleton/>
                  <LoadingSkeleton/>
                  <LoadingSkeleton/></>}
              {
                  data && data.map(d=>{
                      return <PodcastCard podcast={d} key={d.id} />
                  })
              }
          </ScrollView>
      </ThemedView>
    </SafeAreaView>
  );
}

export default HomeScreen;