import io from "socket.io-client";
import useOpmlImport from "../store/opmlImportSlice";
import useCommon from "../store/CommonSlice";
import {components} from "../../schema";
import {decodeHTMLEntities} from "./decodingUtilities";
import {enqueueSnackbar} from "notistack";
import {t} from "i18next";
import {QueryClient} from "@tanstack/react-query";

const socketio = io('/main')
const queryClient = new QueryClient()


socketio.on('offlineAvailable', (data: {
    podcast_episode: components['schemas']['PodcastEpisodeDto']
}) => {
    if (!data) {
        return
    }
    enqueueSnackbar(t('new-podcast-episode-added', {name: decodeHTMLEntities(data.podcast_episode.name)}), {variant: 'success'})
    const queries = queryClient.getQueryCache().getAll()



    for (const cache of queries) {
        console.log(cache.queryKey)
        if (cache.queryKey.length === 3 && cache.queryKey[0] === 'get' && (cache.queryKey[1] as string) === '/api/v1/podcasts/{id}/episodes' && (cache.queryKey[2] as any).params.path.id === data.podcast_episode.podcast_id.toString()) {
            queryClient.setQueryData(cache.queryKey, (oldData: components["schemas"]["PodcastEpisodeWithHistory"][]) => {
                if (!oldData) {
                    return oldData
                }

                console.log(oldData)

                return oldData.map(p => {
                    if (p.podcastEpisode.id === data.podcast_episode.id) {
                        const foundDownload = JSON.parse(JSON.stringify(p)) as components["schemas"]["PodcastEpisodeWithHistory"]

                        foundDownload.podcastEpisode.status = true
                        foundDownload.podcastEpisode.url = data.podcast_episode.url
                        foundDownload.podcastEpisode.local_url = data.podcast_episode.local_url
                        foundDownload.podcastEpisode.image_url = data.podcast_episode.image_url
                        foundDownload.podcastEpisode.local_image_url = data.podcast_episode.local_image_url

                        return foundDownload
                    }

                    return p
                }) satisfies  components["schemas"]["PodcastEpisodeWithHistory"][]
            })
        }
    }
})

socketio.on('opmlError', (data) => {

    useOpmlImport.getState().setProgress([...useOpmlImport.getState().progress, false])
    useOpmlImport.getState().setMessages([...useOpmlImport.getState().messages, data.message])
})

socketio.on('refreshedPodcast', (data) => {
    const podcast = data.podcast

    enqueueSnackbar(t('podcast-refreshed', {name: decodeHTMLEntities(podcast.name)}), {variant: 'success'})
})

socketio.on('addedEpisodes', (data) => {
    enqueueSnackbar(t('new-podcast-episode-added', {name: decodeHTMLEntities(data.podcast.name)}), {variant: 'success'})
})

socketio.on('addedPodcast', (data) => {
    const podcast = data.podcast

    for (const cache of queryClient.getQueryCache().getAll()) {
        if (cache.queryKey[0] === 'get' && (cache.queryKey[1] as string) === '/api/v1/podcasts/search') {
            queryClient.setQueryData(cache.queryKey, (oldData: components["schemas"]["PodcastDto"][]) => {
                return [podcast, ...oldData]
            })
        }
    }
    enqueueSnackbar(t('new-podcast-added', {name: decodeHTMLEntities(podcast.name)}), {variant: 'success'})
})

socketio.on('deletedPodcastEpisodeLocally', (data) => {
    const updatedPodcastEpisodes = useCommon.getState().selectedEpisodes.map(e => {
        if (e.podcastEpisode.episode_id === data.podcast_episode.episode_id) {
            const clonedPodcast = Object.assign({}, data.podcast_episode)

            clonedPodcast.status = false

            return {
                podcastEpisode: clonedPodcast
            }
        }

        return e
    })

    enqueueSnackbar(t('podcast-episode-deleted', {name: decodeHTMLEntities(data.podcast_episode.name)}), {variant: 'success'})
    useCommon.getState().setSelectedEpisodes(updatedPodcastEpisodes)
})

socketio.on('opmlAdded', () => {
    useOpmlImport.getState().setProgress([...useOpmlImport.getState().progress, true])
})


export {queryClient}