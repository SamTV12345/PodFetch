export const getAudioPlayer = () => {
    return document.getElementById('audio-player') as HTMLAudioElement
}


export const startAudioPlayer = async (audioUrl: string, position: number)=>{
    const audioPlayer = getAudioPlayer()
    audioPlayer.src = audioUrl
    audioPlayer.currentTime = position
    audioPlayer.load()
    await audioPlayer.play()
}