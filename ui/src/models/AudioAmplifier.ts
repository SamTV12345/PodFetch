export class AudioAmplifier {
	constructor(private audioElement: HTMLAudioElement) {
		this.source = this.audioContext.createMediaElementSource(this.audioElement)
		this.source.connect(this.gainNode)
		this.gainNode.connect(this.audioContext.destination)
	}
	private audioContext = new AudioContext()
	private gainNode = this.audioContext.createGain()
	private readonly source

	public setVolume(volume: number) {
		this.gainNode.gain.value = volume
	}
	getSource() {
		return this.source
	}

	destroy() {
		this.gainNode.disconnect()
		this.source.disconnect()
	}
}
