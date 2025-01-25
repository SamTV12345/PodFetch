import {client} from "./http";

const wsEndpoint = "ws"

export const configWSUrl = (url: string) => {
    if (url.startsWith("http")) {
        return url.replace("http", "ws") + wsEndpoint
    }
    return url.replace("https", "wss") + wsEndpoint
}
export const logCurrentPlaybackTime = (episodeId: string, timeInSeconds: number) => {
    client.POST("/api/v1/podcasts/episode", {
        body: {
            podcastEpisodeId: episodeId,
            time: Number(timeInSeconds.toFixed(0))
        }
    })
}
