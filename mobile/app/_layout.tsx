import { DarkTheme, DefaultTheme, ThemeProvider } from '@react-navigation/native';
import { useFonts } from 'expo-font';
import { Stack } from 'expo-router';
import * as SplashScreen from 'expo-splash-screen';
import { StatusBar } from 'expo-status-bar';
import { useEffect } from 'react';
import "../i18n/i18n"
import {
  QueryClient,
  QueryClientProvider,
} from '@tanstack/react-query'
import 'react-native-reanimated';

import { useColorScheme } from '@/hooks/useColorScheme';
import {styles} from "@/styles/styles";

// Prevent the splash screen from auto-hiding before asset loading is complete.
SplashScreen.preventAutoHideAsync();


const queryClient = new QueryClient()

export default function RootLayout() {
  const colorScheme = useColorScheme();
  const [loaded] = useFonts({
    SpaceMono: require('../assets/fonts/SpaceMono-Regular.ttf'),
  });

  useEffect(() => {
    if (loaded) {
      SplashScreen.hideAsync();
    }
  }, [loaded]);

  if (!loaded) {
    return null
  }

  return (
      <QueryClientProvider client={queryClient}>
    <ThemeProvider value={{
      ...DarkTheme,
      dark: true,
       colors: {
         ...DarkTheme.colors,
         background: styles.lightDarkColor,
         text: styles.white,
       },
    }}>
      <Stack>
        <Stack.Screen name="(tabs)" options={{ headerShown: false }} />
        <Stack.Screen name="+not-found" />
        <Stack.Screen name="podcasts/[id]/info" options={{headerShown: false}} />
        <Stack.Screen name="podcasts/[id]/details" options={{headerShown: false}} />
        <Stack.Screen name="episodes/[id]/index" options={{headerShown: false}} />
      </Stack>
      <StatusBar style="light" backgroundColor="#000" />
    </ThemeProvider>
      </QueryClientProvider>
  );
}
