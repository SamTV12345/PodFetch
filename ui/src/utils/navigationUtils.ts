import axios from "axios";



const wsEndpoint = "ws"

export const configWSUrl = (url: string) => {
    if (url.startsWith("http")) {
        return url.replace("http", "ws") + wsEndpoint
    }
    return url.replace("https", "wss") + wsEndpoint
}
export const logCurrentPlaybackTime = (episodeId: string, timeInSeconds: number) => {
    axios.post("/podcasts/episode", {
        podcastEpisodeId: episodeId,
        time: Number(timeInSeconds.toFixed(0))
    })
}
