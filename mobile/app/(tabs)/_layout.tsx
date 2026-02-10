import { Tabs, TabList, TabTrigger, TabSlot } from 'expo-router/ui';
import {Text, useColorScheme, View} from "react-native";
import {useTranslation} from "react-i18next";
import {styles} from "@/styles/styles";
import {IconSymbol} from "@/components/ui/IconSymbol";
import {ThemedText} from "@/components/ThemedText";
import {usePathname} from "expo-router";
import {AudioPlayer} from "@/components/AudioPlayer";


export default function TabLayout() {
    const colorScheme = useColorScheme();
    const {t} = useTranslation()
    const pathname = usePathname();


    return (
      <Tabs>
          <TabSlot />
          <AudioPlayer/>
          <TabList style={{
              backgroundColor: styles.darkColor,
              width: '90%',
              marginLeft: 'auto',
              marginRight: 'auto',
              bottom: '3%',
              paddingBottom: 10,
              paddingTop: 10,
              paddingLeft: 20,
              paddingRight: 20,
              borderStyle: undefined,
              borderBottomLeftRadius: 10,
              borderBottomRightRadius: 10,
          }}>
              <TabTrigger name="home" href="/(tabs)" style={{flexDirection: 'column', display: 'flex'}}>
                  <IconSymbol size={20} name={"house.fill"} color={ pathname == "/"? styles.accentColor: 'white'} style={{alignSelf: 'center',}} />
                  <ThemedText style={{color: pathname == "/"? styles.accentColor: 'white', fontSize: 15, marginTop: 'auto',  marginLeft: 'auto', marginBottom: 'auto'}}>{t('home')}</ThemedText>
              </TabTrigger>
              <TabTrigger name="search" href="/(tabs)/search"  style={{flexDirection: 'column', display: 'flex'}}>
                  <IconSymbol size={20} name={"magnifyingglass.circle"} color={ pathname == "/search"? styles.accentColor: 'white'}  style={{alignSelf: 'center'}} />
                  <ThemedText style={{color: pathname == "/search"? styles.accentColor: 'white', fontSize: 15, marginTop: 'auto',  marginLeft: 'auto', marginBottom: 'auto'}}>{t('search')}</ThemedText>
              </TabTrigger>
              <TabTrigger name="library" href="/(tabs)/library"  style={{flexDirection: 'column', display: 'flex'}}>
                  <IconSymbol size={20} name={"bookmark.fill"} color={ pathname == "/library"? styles.accentColor: 'white'}  style={{alignSelf: 'center'}} />
                  <ThemedText style={{color: pathname == "/library"? styles.accentColor: 'white', fontSize: 15, marginTop: 'auto',  marginLeft: 'auto', marginBottom: 'auto'}}>{t('library')}</ThemedText>
              </TabTrigger>
              <TabTrigger name="settings" href="/(tabs)/settings"  style={{flexDirection: 'column', display: 'flex'}}>
                  <IconSymbol size={20} name={"gearshape.fill"} color={ pathname == "/settings"? styles.accentColor: 'white'}  style={{alignSelf: 'center'}} />
                  <ThemedText style={{color: pathname == "/settings"? styles.accentColor: 'white', fontSize: 15, marginTop: 'auto',  marginLeft: 'auto', marginBottom: 'auto'}}>{t('settings')}</ThemedText>
              </TabTrigger>
          </TabList>
      </Tabs>
  );
}
