import { DarkTheme, DefaultTheme, ThemeProvider } from '@react-navigation/native';
import { useFonts } from 'expo-font';
import { Stack, useRouter, useSegments, usePathname } from 'expo-router';
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
import { useAutoSync } from '@/hooks/useSync';
import {styles} from "@/styles/styles";
import { useStore } from '@/store/store';
import { AudioProvider } from '@/components/AudioProvider';
import { AudioPlayer } from '@/components/AudioPlayer';

// Prevent the splash screen from auto-hiding before asset loading is complete.
SplashScreen.preventAutoHideAsync();


const queryClient = new QueryClient()

export default function RootLayout() {
  const colorScheme = useColorScheme();
  const router = useRouter();
  const segments = useSegments();
  const pathname = usePathname();
  const serverUrl = useStore((state) => state.serverUrl);

  // Prüfe ob wir auf einer Tab-Seite sind (wo die TabBar sichtbar ist)
  const isTabScreen = segments[0] === '(tabs)';
  // Auf Tab-Seiten brauchen wir mehr Abstand für die TabBar
  const audioPlayerBottomOffset = isTabScreen ? 85 : 30;

  // Starte automatische Synchronisation für Offline-Daten
  useAutoSync(30000); // Alle 30 Sekunden prüfen

  const [loaded] = useFonts({
    SpaceMono: require('../assets/fonts/SpaceMono-Regular.ttf'),
  });

  useEffect(() => {
    if (loaded) {
      SplashScreen.hideAsync();
    }
  }, [loaded]);

  // Redirect based on server URL configuration
  useEffect(() => {
    if (!loaded) return;

    const inServerSetup = segments[0] === 'server-setup';

    if (!serverUrl && !inServerSetup) {
      // No server configured, redirect to setup
      router.replace('/server-setup');
    } else if (serverUrl && inServerSetup) {
      // Server configured, redirect to main app
      router.replace('/(tabs)');
    }
  }, [serverUrl, segments, loaded]);

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
      <AudioProvider>
      <Stack>
        <Stack.Screen name="server-setup" options={{ headerShown: false }} />
        <Stack.Screen name="(tabs)" options={{ headerShown: false }} />
        <Stack.Screen name="+not-found" />
        <Stack.Screen name="podcasts/[id]/info" options={{headerShown: false}} />
        <Stack.Screen name="podcasts/[id]/details" options={{headerShown: false}} />
        <Stack.Screen name="episodes/[id]/index" options={{headerShown: false}} />
        <Stack.Screen name="player" options={{headerShown: false, presentation: 'modal'}} />
      </Stack>
      {/* Globaler AudioPlayer - nur anzeigen wenn nicht im Player-Modal oder Server-Setup */}
      {pathname !== '/player' && pathname !== '/server-setup' && (
        <AudioPlayer bottomOffset={audioPlayerBottomOffset} />
      )}
      </AudioProvider>
      <StatusBar style="light" backgroundColor="#000" />
    </ThemeProvider>
      </QueryClientProvider>
  );
}
