import {$api} from "../utils/http";

export const usePlaybackLogger = () => {
    const logPlaybackMutation = $api.useMutation('post', '/api/v1/podcasts/episode')

    return (episodeId: string, timeInSeconds: number) => {
        logPlaybackMutation.mutate({
            body: {
                podcastEpisodeId: episodeId,
                time: Number(timeInSeconds.toFixed(0))
            }
        })
    }
}
