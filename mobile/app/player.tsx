import {Image, Modal, Pressable, ScrollView, Text, View, Share} from "react-native";
import {SafeAreaView} from "react-native-safe-area-context";
import {styles} from "@/styles/styles";
import {useStore} from "@/store/store";
import {AntDesign, MaterialIcons, Ionicons, Feather} from "@expo/vector-icons";
import {useAudioPlayerStatus} from 'expo-audio';
import {router} from 'expo-router';
import {useTranslation} from "react-i18next";
import Slider from '@react-native-community/slider';
import {useState, useCallback} from 'react';

const formatTime = (seconds: number): string => {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
};

export default function PlayerScreen() {
    const {t} = useTranslation();
    const selectedPodcastEpisode = useStore(state => state.podcastEpisodeRecord);
    const audioPlayer = useStore(state => state.audioPlayer);
    const isPlaying = useStore(state => state.isPlaying);
    const setIsPlaying = useStore(state => state.setIsPlaying);

    const status = audioPlayer ? useAudioPlayerStatus(audioPlayer) : null;

    const [isOptionsVisible, setIsOptionsVisible] = useState(false);
    const [isSeeking, setIsSeeking] = useState(false);
    const [seekValue, setSeekValue] = useState(0);
    const [isToggling, setIsToggling] = useState(false); // Verhindert Doppelklicks

    const handleSeekStart = useCallback(() => {
        setIsSeeking(true);
        setSeekValue(status?.currentTime || 0);
    }, [status?.currentTime]);

    const handleSeekChange = useCallback((value: number) => {
        setSeekValue(value);
    }, []);

    const handleSeekComplete = useCallback((value: number) => {
        if (audioPlayer) {
            audioPlayer.seekTo(value);
        }
        setIsSeeking(false);
    }, [audioPlayer]);

    const handleShare = useCallback(async () => {
        if (selectedPodcastEpisode) {
            try {
                await Share.share({
                    message: t('share-podcast', {
                        podcast: selectedPodcastEpisode.podcastEpisode.name,
                        url: selectedPodcastEpisode.podcastEpisode.url
                    }),
                });
            } catch (error) {
                console.error('Error sharing:', error);
            }
        }
        setIsOptionsVisible(false);
    }, [selectedPodcastEpisode, t]);

    const handleSkipBackward = () => {
        if (audioPlayer && status) {
            const newTime = Math.max(0, status.currentTime - 15);
            audioPlayer.seekTo(newTime);
        }
    };

    const handleSkipForward = () => {
        if (audioPlayer && status) {
            const newTime = Math.min(status.duration, status.currentTime + 30);
            audioPlayer.seekTo(newTime);
        }
    };

    const handleTogglePlay = useCallback(() => {
        // Verhindere Doppelklicks
        if (isToggling || !audioPlayer) return;

        setIsToggling(true);

        if (isPlaying) {
            audioPlayer.pause();
            setIsPlaying(false);
        } else {
            audioPlayer.play();
            setIsPlaying(true);
        }

        // Erlaube erneutes Klicken nach 300ms
        setTimeout(() => setIsToggling(false), 300);
    }, [isPlaying, audioPlayer, setIsPlaying, isToggling]);

    if (!selectedPodcastEpisode) {
        return (
            <SafeAreaView style={{flex: 1, backgroundColor: styles.lightDarkColor}}>
                <View style={{flex: 1, justifyContent: 'center', alignItems: 'center'}}>
                    <Text style={{color: 'white'}}>{t('no-episode-selected')}</Text>
                </View>
            </SafeAreaView>
        );
    }

    return (
        <SafeAreaView style={{flex: 1, backgroundColor: styles.lightDarkColor}}>
            {/* Header */}
            <View style={{
                flexDirection: 'row',
                alignItems: 'center',
                paddingHorizontal: 20,
                paddingTop: 10,
            }}>
                <Pressable onPress={() => router.back()}>
                    <MaterialIcons name="keyboard-arrow-down" size={35} color="white"/>
                </Pressable>
                <View style={{flex: 1, alignItems: 'center'}}>
                    <Text style={{color: styles.gray, fontSize: 12}}>{t('now-playing')}</Text>
                    <Text style={{color: 'white', fontSize: 14}} numberOfLines={1}>
                        {selectedPodcastEpisode.podcast?.name || ''}
                    </Text>
                </View>
                <Pressable onPress={() => setIsOptionsVisible(true)}>
                    <MaterialIcons name="more-vert" size={24} color="white"/>
                </Pressable>
            </View>

            {/* Album Art */}
            <View style={{flex: 1, justifyContent: 'center', alignItems: 'center', paddingHorizontal: 40}}>
                <Image
                    source={{uri: selectedPodcastEpisode.podcastEpisode.local_image_url}}
                    style={{
                        width: 300,
                        height: 300,
                        borderRadius: 10,
                        shadowColor: '#000',
                        shadowOffset: {width: 0, height: 10},
                        shadowOpacity: 0.5,
                        shadowRadius: 20,
                    }}
                />
            </View>

            {/* Episode Info */}
            <View style={{paddingHorizontal: 30, marginBottom: 20}}>
                <Text style={{color: 'white', fontSize: 22, fontWeight: 'bold'}} numberOfLines={2}>
                    {selectedPodcastEpisode.podcastEpisode.name}
                </Text>
                <Text style={{color: styles.gray, fontSize: 16, marginTop: 5}}>
                    {selectedPodcastEpisode.podcast?.name || ''}
                </Text>
            </View>

            {/* Progress Bar */}
            <View style={{paddingHorizontal: 20}}>
                <Slider
                    style={{width: '100%', height: 40}}
                    minimumValue={0}
                    maximumValue={status?.duration || 1}
                    value={isSeeking ? seekValue : (status?.currentTime || 0)}
                    onSlidingStart={handleSeekStart}
                    onValueChange={handleSeekChange}
                    onSlidingComplete={handleSeekComplete}
                    minimumTrackTintColor={styles.accentColor}
                    maximumTrackTintColor={styles.gray}
                    thumbTintColor="white"
                />
                <View style={{flexDirection: 'row', justifyContent: 'space-between', marginTop: -5, paddingHorizontal: 10}}>
                    <Text style={{color: styles.gray, fontSize: 12}}>
                        {formatTime(isSeeking ? seekValue : (status?.currentTime || 0))}
                    </Text>
                    <Text style={{color: styles.gray, fontSize: 12}}>
                        -{formatTime((status?.duration || 0) - (isSeeking ? seekValue : (status?.currentTime || 0)))}
                    </Text>
                </View>
            </View>

            {/* Controls */}
            <View style={{
                flexDirection: 'row',
                justifyContent: 'center',
                alignItems: 'center',
                paddingVertical: 30,
                gap: 40,
            }}>
                {/* Skip Backward 15s */}
                <Pressable onPress={handleSkipBackward} style={{alignItems: 'center'}}>
                    <Ionicons name="play-back" size={35} color="white"/>
                    <Text style={{color: styles.gray, fontSize: 10, marginTop: 2}}>15</Text>
                </Pressable>

                {/* Play/Pause */}
                <Pressable
                    onPress={handleTogglePlay}
                    style={{
                        backgroundColor: 'white',
                        width: 70,
                        height: 70,
                        borderRadius: 35,
                        justifyContent: 'center',
                        alignItems: 'center',
                    }}
                >
                    <AntDesign
                        name={isPlaying ? "pause" : "caret-right"}
                        size={30}
                        color={styles.darkColor}
                    />
                </Pressable>

                {/* Skip Forward 30s */}
                <Pressable onPress={handleSkipForward} style={{alignItems: 'center'}}>
                    <Ionicons name="play-forward" size={35} color="white"/>
                    <Text style={{color: styles.gray, fontSize: 10, marginTop: 2}}>30</Text>
                </Pressable>
            </View>

            {/* Bottom spacing */}
            <View style={{height: 30}}/>

            {/* Options Modal */}
            <Modal
                animationType="slide"
                transparent={true}
                visible={isOptionsVisible}
                onRequestClose={() => setIsOptionsVisible(false)}
            >
                <Pressable
                    style={{flex: 1, backgroundColor: 'rgba(0,0,0,0.5)'}}
                    onPress={() => setIsOptionsVisible(false)}
                >
                    <Pressable
                        style={{
                            position: 'absolute',
                            bottom: 0,
                            left: 0,
                            right: 0,
                            backgroundColor: styles.darkColor,
                            borderTopLeftRadius: 20,
                            borderTopRightRadius: 20,
                            maxHeight: '80%',
                        }}
                        onPress={(e) => e.stopPropagation()}
                    >
                        {/* Modal Header */}
                        <View style={{
                            flexDirection: 'row',
                            alignItems: 'center',
                            padding: 20,
                            borderBottomWidth: 1,
                            borderBottomColor: styles.gray,
                        }}>
                            <Image
                                source={{uri: selectedPodcastEpisode.podcastEpisode.local_image_url}}
                                style={{width: 50, height: 50, borderRadius: 5}}
                            />
                            <View style={{flex: 1, marginLeft: 15}}>
                                <Text style={{color: 'white', fontSize: 16, fontWeight: 'bold'}} numberOfLines={1}>
                                    {selectedPodcastEpisode.podcastEpisode.name}
                                </Text>
                                <Text style={{color: styles.gray, fontSize: 14}} numberOfLines={1}>
                                    {selectedPodcastEpisode.podcast?.name || ''}
                                </Text>
                            </View>
                            <Pressable onPress={() => setIsOptionsVisible(false)}>
                                <AntDesign name="close" size={24} color="white"/>
                            </Pressable>
                        </View>

                        <ScrollView style={{maxHeight: 400}}>
                            {/* Description Section */}
                            <View style={{padding: 20}}>
                                <Text style={{color: 'white', fontSize: 18, fontWeight: 'bold', marginBottom: 10}}>
                                    {t('description')}
                                </Text>
                                <Text style={{color: styles.whiteSubText, fontSize: 14, lineHeight: 22}}>
                                    {selectedPodcastEpisode.podcastEpisode.description || t('no-description')}
                                </Text>
                            </View>

                            {/* Options */}
                            <View style={{paddingHorizontal: 20, paddingBottom: 40}}>
                                {/* Share */}
                                <Pressable
                                    style={{
                                        flexDirection: 'row',
                                        alignItems: 'center',
                                        paddingVertical: 15,
                                        borderTopWidth: 1,
                                        borderTopColor: styles.gray,
                                    }}
                                    onPress={handleShare}
                                >
                                    <Feather name="share" size={22} color="white"/>
                                    <Text style={{color: 'white', fontSize: 16, marginLeft: 15}}>
                                        {t('share')}
                                    </Text>
                                </Pressable>

                                {/* Go to Podcast */}
                                <Pressable
                                    style={{
                                        flexDirection: 'row',
                                        alignItems: 'center',
                                        paddingVertical: 15,
                                        borderTopWidth: 1,
                                        borderTopColor: styles.gray,
                                    }}
                                    onPress={() => {
                                        setIsOptionsVisible(false);
                                        router.back();
                                        router.push({
                                            pathname: '/podcasts/[id]/details',
                                            params: {id: selectedPodcastEpisode.podcastEpisode.podcast_id}
                                        });
                                    }}
                                >
                                    <MaterialIcons name="podcasts" size={22} color="white"/>
                                    <Text style={{color: 'white', fontSize: 16, marginLeft: 15}}>
                                        {t('go-to-podcast')}
                                    </Text>
                                </Pressable>

                                {/* Episode Info */}
                                <View style={{
                                    flexDirection: 'row',
                                    alignItems: 'center',
                                    paddingVertical: 15,
                                    borderTopWidth: 1,
                                    borderTopColor: styles.gray,
                                }}>
                                    <AntDesign name="info-circle" size={22} color="white"/>
                                    <View style={{marginLeft: 15}}>
                                        <Text style={{color: 'white', fontSize: 16}}>
                                            {t('episode-info')}
                                        </Text>
                                        <Text style={{color: styles.gray, fontSize: 12, marginTop: 2}}>
                                            {t('duration')}: {formatTime(selectedPodcastEpisode.podcastEpisode.total_time)}
                                        </Text>
                                        <Text style={{color: styles.gray, fontSize: 12}}>
                                            {t('published')}: {selectedPodcastEpisode.podcastEpisode.date_of_recording}
                                        </Text>
                                    </View>
                                </View>
                            </View>
                        </ScrollView>
                    </Pressable>
                </Pressable>
            </Modal>
        </SafeAreaView>
    );
}
