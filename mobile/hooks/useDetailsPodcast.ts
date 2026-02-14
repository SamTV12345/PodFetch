import {useLocalSearchParams} from "expo-router";
import {$api} from "@/client";
import {useEffect} from "react";
import {useStore} from "@/store/store";

export const useDetailsPodcast = ()=>{
    const { id } = useLocalSearchParams();

    const podcastDetailedData = $api.useQuery('get', '/api/v1/podcasts/{id}', {
        params: {
            path: {
                id: id as string
            }
        }
    })
    const dataEpisodes = $api.useQuery('get', '/api/v1/podcasts/{id}/episodes', {
        params: {
            path: {
                id: id as string
            }
        }
    })

    const updateFavored = $api.useMutation("put", "/api/v1/podcasts/favored", {
        onSuccess: () => {
            podcastDetailedData.refetch()
        }
    })

    useEffect(() => {
        if (podcastDetailedData.data) {
            useStore.getState().savePodcast(podcastDetailedData.data.id.toString(), podcastDetailedData.data)
        }
    }, [podcastDetailedData]);


    return {
        podcastDetailedData,
        dataEpisodes,
        updateFavored
    }
}
