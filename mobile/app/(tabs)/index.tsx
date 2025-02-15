import { ThemedView } from '@/components/ThemedView';
import {SafeAreaView} from "react-native-safe-area-context";
import {styles} from "@/styles/styles";
import Heading1 from "@/components/text/Heading1";
import {$api} from "@/client";
import {useTranslation} from "react-i18next";

import SkeletonLoading from 'expo-skeleton-loading'
import {ScrollView, View} from "react-native";
import Heading2 from "@/components/text/Heading2";
import {LoadingSkeleton} from "@/components/ui/LoadingSkeleton";

export default function HomeScreen() {
  const {data, isError, isLoading} = $api.useQuery('get', '/api/v1/podcasts')
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
          <ScrollView horizontal={true} style={{paddingBottom: 20}} overScrollMode="never">
              <LoadingSkeleton/>
              <LoadingSkeleton/>
              <LoadingSkeleton/>
              <LoadingSkeleton/>
              <LoadingSkeleton/>
              <LoadingSkeleton/>
          </ScrollView>

          <Heading2 more onMore={()=>{}}>{t('your-podcasts')}</Heading2>
          <ScrollView horizontal={true} style={{paddingBottom: 20}} overScrollMode="never">
              <LoadingSkeleton/>
              <LoadingSkeleton/>
              <LoadingSkeleton/>
              <LoadingSkeleton/>
              <LoadingSkeleton/>
              <LoadingSkeleton/>
          </ScrollView>
      </ThemedView>
    </SafeAreaView>
  );
}
