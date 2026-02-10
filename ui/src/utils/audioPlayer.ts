export const getAudioPlayer = () => {
    return document.getElementById('audio-player') as HTMLAudioElement
}


export const startAudioPlayer = async (audioUrl: string, position: number)=>{
    console.log("Starting audio " + audioUrl + " at position " + position)
    const audioPlayer = getAudioPlayer()
    audioPlayer.pause()
    audioPlayer.src = audioUrl
    audioPlayer.currentTime = position
    audioPlayer.load()
    await audioPlayer.play()
}