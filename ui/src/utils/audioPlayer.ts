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
    const audioPlayer = getAudioPlayer()
    if (!audioPlayer) {
        return
    }
    const safePosition = Number.isFinite(position) && position > 0 ? position : 0
    let positionApplied = false
    const applyPosition = () => {
        if (positionApplied) {
            return
        }
        try {
            const duration = Number.isFinite(audioPlayer.duration) ? audioPlayer.duration : Infinity
            audioPlayer.currentTime = Math.min(safePosition, duration)
            positionApplied = true
        } catch {
            audioPlayer.currentTime = 0
            positionApplied = true
        }
    }

    audioPlayer.pause()
    audioPlayer.src = audioUrl
    audioPlayer.load()

    if (audioPlayer.readyState >= 1) {
        applyPosition()
    } else {
        const onReady = () => applyPosition()
        audioPlayer.addEventListener('loadedmetadata', onReady, {once: true})
        audioPlayer.addEventListener('canplay', onReady, {once: true})
        setTimeout(() => applyPosition(), 1200)
    }

    // Trigger play immediately to keep browser user-gesture context.
    try {
        await audioPlayer.play()
    } catch {
        // Retry once after media becomes playable.
        await new Promise((resolve) => setTimeout(resolve, 150))
        try {
            await audioPlayer.play()
        } catch {
            audioPlayer.muted = false
            if (audioPlayer.volume <= 0) {
                audioPlayer.volume = 1
            }
            try {
                await audioPlayer.play()
            } catch {
                // Keep silent here; user can retry with play button.
            }
        }
    }
}
