export class AudioAmplifier {
    constructor(audioElement) {
        this.source = this.audioContext.createMediaElementSource(audioElement);
        this.source.connect(this.gainNode);
        this.gainNode.connect(this.audioContext.destination);
    }

    audioContext = new AudioContext();
    gainNode = this.audioContext.createGain();
    source;

    setVolume(volume) {
        this.gainNode.gain.value = volume;
    }

    getSource() {
        return this.source;
    }


    loadSource(url) {
        this.source.mediaElement.src = url;
    }

    destroy() {
        this.gainNode.disconnect()
        this.source.disconnect()
    }
}

let audioPlayerInstance;


export const getAudioPlayerInstance = () => {
    if (!audioPlayerInstance) {
        audioPlayerInstance = new AudioAmplifier(document.getElementById('main-audio'))
    }
    return audioPlayerInstance;
}

export default audioPlayerInstance