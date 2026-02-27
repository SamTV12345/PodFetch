export const getAudioPlayer = () => {
    return document.getElementById('audio-player') as HTMLMediaElement | null
}

export const isVideoUrl = (url?: string | null): boolean => {
    if (!url) return false
    const cleanPath = url.split('?')[0] || ''
    const ext = cleanPath.split('.').pop()?.toLowerCase() || ''
    return ['mp4', 'm4v', 'mov', 'webm'].includes(ext)
}

export const startAudioPlayer = async (audioUrl: string, position: number) => {
    console.log("Starting media " + audioUrl + " at position " + position)
    const audioPlayer = getAudioPlayer()
    if (!audioPlayer) {
        return
    }
    audioPlayer.pause()
    audioPlayer.src = audioUrl
    audioPlayer.currentTime = position
    audioPlayer.load()
    await audioPlayer.play()
}
