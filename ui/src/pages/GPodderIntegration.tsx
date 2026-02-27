import {useTranslation} from "react-i18next";
import {handleAddPodcast} from "../utils/ErrorSnackBarResponses";
import {CustomButtonPrimary} from "../components/CustomButtonPrimary";
import {$api} from "../utils/http";
import {components} from "../../schema";
import {useQueryClient} from "@tanstack/react-query";
import {LoadingSkeletonSpan} from "../components/ui/LoadingSkeletonSpan";

export const GPodderIntegration = ()=> {
    const {t} = useTranslation()
    const queryClient = useQueryClient()
    const gpodder = $api.useQuery('get', '/api/v1/podcasts/available/gpodder')
    const addPodcastMutation = $api.useMutation('post', '/api/v1/podcasts/feed')


    const addPodcast = (feedUrl: string)=>{
        queryClient.setQueryData(['get', '/api/v1/podcasts/available/gpodder'], (oldData?: components["schemas"]["GPodderAvailablePodcasts"][])=>{
           return oldData?.filter(d=>d.podcast!=feedUrl)
        })
        addPodcastMutation.mutateAsync({
            body: {
                rssFeedUrl: feedUrl
            }
        }).then((v: any) => {
            handleAddPodcast(200, v.name, t)
        })
    }


    return <table className="text-left text-sm w-full overflow-y-auto text-(--fg-color)">
        <thead>
        <tr className="border-b border-stone-300">
            <th scope="col" className="pr-2 py-3 text-(--fg-color)">
                #
            </th>
            <th scope="col" className="px-2 py-3 text-(--fg-color)">
                {t('device')}
            </th>
            <th scope="col" className="px-2 py-3 text-(--fg-color)">
                {t('podcasts')}
            </th>
            <th scope="col" className="px-2 py-3 text-(--fg-color)">
                {t('actions')}
            </th>
        </tr>
        </thead>
        <tbody className="">
        {(gpodder.isLoading || !gpodder.data)? Array.from({length: 5}).map((_, index)=><tr key={index}>
                <td className="px-2 py-4 text-(--fg-color)"><LoadingSkeletonSpan height="30px" loading={gpodder.isLoading}/></td>
                <td className="px-2 py-4  text-(--fg-color)"><LoadingSkeletonSpan height="30px" loading={gpodder.isLoading}/></td>
                <td className="px-2 py-4  text-(--fg-color)"><LoadingSkeletonSpan height="30px" loading={gpodder.isLoading}/></td>
                <td><CustomButtonPrimary><LoadingSkeletonSpan height="30px" loading={gpodder.isLoading} width="100px"/></CustomButtonPrimary></td>
            </tr>):
            gpodder.data.map((int, index)=>{
                return <tr key={index}>
                    <td className="px-2 py-4 text-(--fg-color)">{index}</td>
                    <td className="px-2 py-4  text-(--fg-color)">{int.device}</td>
                    <td className="px-2 py-4  text-(--fg-color)">{int.podcast}</td>
                    <td><CustomButtonPrimary onClick={()=>addPodcast(int.podcast)}>{t('add')}</CustomButtonPrimary></td>
                </tr>
                }
            )
        }
        </tbody>
    </table>
}
