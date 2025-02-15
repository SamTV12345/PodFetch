import { Tabs } from 'expo-router';
import React from 'react';
import {Platform} from 'react-native';

import { HapticTab } from '@/components/HapticTab';
import { IconSymbol } from '@/components/ui/IconSymbol';
import TabBarBackground from '@/components/ui/TabBarBackground';
import { Colors } from '@/constants/Colors';
import { useColorScheme } from '@/hooks/useColorScheme';
import {styles} from "@/styles/styles";

import {useTranslation} from "react-i18next";
import {Header} from "@react-navigation/elements";

export default function TabLayout() {
  const colorScheme = useColorScheme();
  const {t} = useTranslation()

  return (
    <Tabs
      screenOptions={{
        tabBarActiveTintColor: Colors[colorScheme ?? 'light'].tint,
        headerShown: true,
          header: ()=><Header title={"test"}/>,
          tabBarButton: HapticTab,
        tabBarBackground: TabBarBackground,
          tabBarStyle: Platform.select({
              ios: {
                  position: 'absolute',
                  bottom: 20,
                  left: '10%', // Adjust this value to create the desired margin
                  right: '10%', // Adjust this value to create the desired margin
                  backgroundColor: styles.darkColor,
                  borderRadius: 20,
                  shadowColor: '#000',
                  shadowOffset: { width: 0, height: 2 },
                  shadowOpacity: 0.3,
                  shadowRadius: 4,
              },
              android: {
                  position: 'absolute',
                  bottom: 20,
                  left: 50,
                  right: 20,
                  backgroundColor: styles.darkColor,
                  borderRadius: 20,
                  elevation: 5,
              }
          }),
      }}>
        <Tabs.Screen
            name={"index"}
            options={{
                tabBarActiveTintColor: styles.accentColor,
                headerShown: false,
                tabBarButton: HapticTab,
                tabBarBackground: TabBarBackground,
                tabBarStyle: Platform.select({ios: {
                        position: 'absolute',
                        bottom: 20,
                        backgroundColor: 'rgb(22 21 20)',
                        borderRadius: 20,
                        shadowColor: '#000',
                        shadowOffset: { width: 0, height: 2 },
                        shadowOpacity: 0.3,
                        shadowRadius: 4,
                    },
                    android: {
                        position: 'absolute',
                        bottom: 20,
                        left: '10%',
                        right: '10%',
                        backgroundColor: 'rgb(22 21 20)',
                        borderRadius: 20, // Optional: Add rounded corners
                        elevation: 5, // Optional: Add elevation for Android
                    }}),
                title: t('home'),
                tabBarIcon: ({ color }) => <IconSymbol size={28} name={"house.fill"} color={color} />,
            }}
        />
        <Tabs.Screen
            name={"search"}
            options={{
                header: ()=><Header title={"test"}/>,
                tabBarActiveTintColor: styles.accentColor,
                headerShown: false,
                tabBarButton: HapticTab,
                tabBarBackground: TabBarBackground,
                tabBarStyle: Platform.select({ios: {
                        position: 'absolute',
                        bottom: 20,
                        backgroundColor: 'rgb(22 21 20)',
                        borderRadius: 20,
                        shadowColor: '#000',
                        shadowOffset: { width: 0, height: 2 },
                        shadowOpacity: 0.3,
                        shadowRadius: 4,
                    },
                    android: {
                        position: 'absolute',
                        bottom: 20,
                        left: '10%',
                        right: '10%',
                        backgroundColor: 'rgb(22 21 20)',
                        borderRadius: 20, // Optional: Add rounded corners
                        elevation: 5, // Optional: Add elevation for Android
                    }}),
                title: t('search'),
                tabBarIcon: ({ color }) => <IconSymbol size={28} name={"magnifyingglass.circle"} color={color} />,
            }}
        />
        <Tabs.Screen
            name={"library"}
            options={{
                tabBarActiveTintColor: styles.accentColor,
                headerShown: false,
                tabBarButton: HapticTab,
                tabBarBackground: TabBarBackground,
                tabBarStyle: Platform.select({ios: {
                        position: 'absolute',
                        bottom: 20,
                        backgroundColor: 'rgb(22 21 20)',
                        borderRadius: 20,
                        shadowColor: '#000',
                        shadowOffset: { width: 0, height: 2 },
                        shadowOpacity: 0.3,
                        shadowRadius: 4,
                    },
                    android: {
                        position: 'absolute',
                        bottom: 20,
                        left: '10%',
                        right: '10%',
                        backgroundColor: 'rgb(22 21 20)',
                        borderRadius: 40,
                        elevation: 5,
                    }}),
                title: t('library'),
                tabBarIcon: ({ color }) => <IconSymbol size={28} name={"bookmark.fill"} color={color} />,
            }}
        />
    </Tabs>
  );
}
