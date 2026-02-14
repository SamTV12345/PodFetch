import { Tabs, TabList, TabTrigger, TabSlot } from 'expo-router/ui';
import {useTranslation} from "react-i18next";
import {styles} from "@/styles/styles";
import {IconSymbol} from "@/components/ui/IconSymbol";
import {ThemedText} from "@/components/ThemedText";
import {router, usePathname} from "expo-router";
import React, {useCallback, useState} from 'react';
import {Image, Pressable, Text, useWindowDimensions, View} from 'react-native';
import { useSafeAreaInsets } from 'react-native-safe-area-context';
import {Ionicons} from "@expo/vector-icons";
import {useStore} from "@/store/store";


export default function TabLayout() {
    const {t} = useTranslation()
    const pathname = usePathname();
    const insets = useSafeAreaInsets();
    const audioPercent = useStore(state => state.audioProgress);
    const audioPlayer = useStore(state => state.audioPlayer);

    const FALLBACK_TAB_BAR_HEIGHT = 64; // px
    const [tabBarHeight, setTabBarHeight] = useState<number>(FALLBACK_TAB_BAR_HEIGHT);
    const handleOpenFullscreen = () => {
        router.push('/player');
    };
    const selectedPodcastEpisode = useStore(state => state.podcastEpisodeRecord);
    const { width: screenWidth } = useWindowDimensions();
    const setIsPlaying = useStore(state => state.setIsPlaying);

    const isSmallScreen = screenWidth < 375;
    const albumArtSize = isSmallScreen ? 40 : 48;
    const playButtonSize = isSmallScreen ? 32 : 36;
    const iconSize = isSmallScreen ? 18 : 20;
    const titleFontSize = isSmallScreen ? 13 : 14;
    const subtitleFontSize = isSmallScreen ? 11 : 12;
    const isPlaying = useStore(state => state.isPlaying);
    const [isToggling, setIsToggling] = useState(false);

    const handleTogglePlay = useCallback((e: any) => {
        e.stopPropagation();

        if (isToggling || !audioPlayer) return;
        setIsToggling(true);

        if (isPlaying) {
            audioPlayer.pause();
            setIsPlaying(false);
        } else {
            audioPlayer.play();
            setIsPlaying(true);
        }

        setTimeout(() => setIsToggling(false), 300);
    }, [isPlaying, audioPlayer, setIsPlaying, isToggling]);

    // New compact styles used by TabTrigger children to avoid label overflow
    const tabTriggerCommonStyle = {
        flex: 1,
        alignItems: 'center' as const,
        justifyContent: 'center' as const,
        flexDirection: 'column' as const,
        paddingVertical: 6,
        paddingHorizontal: 6, // give each tab a bit of horizontal breathing room
    };

    return (
      <Tabs>
          {selectedPodcastEpisode ? (
              <View
                  style={{
                      position: 'absolute',
                      left: 0,
                      width: '100%',
                      right: 0,
                      bottom: (tabBarHeight),
                      zIndex: 999,
                      elevation: 20,
                      alignItems: 'center',
                      pointerEvents: 'box-none',
                  }}
                  pointerEvents="box-none"
              >
                  <Pressable
                      onPress={handleOpenFullscreen}
                      style={{
                          width: '100%',
                          backgroundColor: styles.darkColor,
                          overflow: 'hidden',
                          shadowColor: '#000',
                          shadowOffset: { width: 0, height: 4 },
                          shadowOpacity: 0.3,
                          shadowRadius: 8,
                          elevation: 8,
                      }}
                  >
                      {/* Progress Bar - oben */}
                      <View style={{
                          height: 3,
                          backgroundColor: styles.gray,
                          width: '100%',
                      }}>
                          <View style={{
                              height: '100%',
                              width: `${audioPercent}%`,
                              backgroundColor: styles.accentColor,
                          }}/>
                      </View>

                      <View style={{
                          flexDirection: 'row',
                          alignItems: 'center',
                          paddingVertical: 8,
                          paddingHorizontal: 8,
                      }}>
                          {/* Album Art */}
                          <Image
                              source={{uri: selectedPodcastEpisode.podcastEpisode.local_image_url}}
                              style={{
                                  width: albumArtSize,
                                  height: albumArtSize,
                                  borderRadius: 6,
                              }}
                          />

                          {/* Text Content */}
                          <View style={{
                              flex: 1,
                              marginLeft: isSmallScreen ? 10 : 12,
                              marginRight: isSmallScreen ? 10 : 12,
                              justifyContent: 'center',
                          }}>
                              <Text
                                  style={{
                                      color: 'white',
                                      fontSize: titleFontSize,
                                      fontWeight: '600',
                                  }}
                                  numberOfLines={1}
                              >
                                  {selectedPodcastEpisode.podcastEpisode.name}
                              </Text>
                              <Text
                                  style={{
                                      color: styles.gray,
                                      fontSize: subtitleFontSize,
                                      marginTop: 2,
                                  }}
                                  numberOfLines={1}
                              >
                                  {selectedPodcastEpisode.podcast?.name || ''}
                              </Text>
                          </View>

                          {/* Controls */}
                          <View style={{
                              flexDirection: 'row',
                              alignItems: 'center',
                              gap: isSmallScreen ? 12 : 16,
                          }}>
                              {/* Play/Pause Button */}
                              <Pressable
                                  onPress={handleTogglePlay}
                                  style={{
                                      width: playButtonSize,
                                      height: playButtonSize,
                                      borderRadius: playButtonSize / 2,
                                      backgroundColor: 'white',
                                      justifyContent: 'center',
                                      alignItems: 'center',
                                  }}
                              >
                                  {isPlaying ? (
                                      <Ionicons name="pause" size={iconSize} color={styles.darkColor} />
                                  ) : (
                                      <Ionicons name="play" size={iconSize} color={styles.darkColor} style={{marginLeft: 2}} />
                                  )}
                              </Pressable>
                          </View>
                      </View>
                  </Pressable>
              </View>
          ) : null}
          <TabSlot />


          <TabList
            // Messe die tatsächliche Höhe der TabList, damit wir paddingBottom genau setzen können
            onLayout={(e) => {
              const h = e.nativeEvent.layout.height;
              if (h && h !== tabBarHeight) setTabBarHeight(h);
            }}
            style={{
              backgroundColor: styles.darkColor,
              width: '100%',
              // keep tablist centered and away from edges
              paddingTop: 10,
              paddingHorizontal: 20, // increased so labels won't touch screen edges
              borderStyle: undefined,
              position: 'absolute',
              left: 0,
              right: 0,
              bottom: 0,
              paddingBottom: insets.bottom,
            }}
          >
              <TabTrigger name="home" href="/(tabs)" style={tabTriggerCommonStyle}>
                  <IconSymbol size={20} name={"house.fill"} color={ pathname == "/"? styles.accentColor: 'white'} style={{alignSelf: 'center'}} />
                  <ThemedText style={{color: pathname == "/"? styles.accentColor: 'white', fontSize: 15, marginTop: 4, alignSelf: 'center'}}>{t('home')}</ThemedText>
              </TabTrigger>
              <TabTrigger name="downloads" href="/(tabs)/downloads"  style={tabTriggerCommonStyle}>
                  <IconSymbol size={20} name={"arrow.down.circle.fill"} color={ pathname == "/downloads"? styles.accentColor: 'white'}  style={{alignSelf: 'center'}} />
                  <ThemedText style={{color: pathname == "/downloads"? styles.accentColor: 'white', fontSize: 15, marginTop: 4, alignSelf: 'center'}}>{t('downloads')}</ThemedText>
              </TabTrigger>
              <TabTrigger name="settings" href="/(tabs)/settings"  style={tabTriggerCommonStyle}>
                  <IconSymbol size={20} name={"gearshape.fill"} color={ pathname == "/settings"? styles.accentColor: 'white'}  style={{alignSelf: 'center'}} />
                  <ThemedText style={{color: pathname == "/settings"? styles.accentColor: 'white', fontSize: 15, marginTop: 4, alignSelf: 'center'}}>{t('settings')}</ThemedText>
              </TabTrigger>
          </TabList>
      </Tabs>
  );
}
