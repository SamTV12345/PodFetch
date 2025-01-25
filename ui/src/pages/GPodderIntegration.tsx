import {Setting} from "../models/Setting";
import {useEffect, useState} from "react";
import {useTranslation} from "react-i18next";
import {Podcast} from "../store/CommonSlice";
import {handleAddPodcast} from "../utils/ErrorSnackBarResponses";
import {CustomButtonPrimary} from "../components/CustomButtonPrimary";
import {client} from "../utils/http";
import {components} from "../../schema";

type GPodderIntegrationItem = {
    device: string,
    podcast: string
}


export const GPodderIntegration = ()=> {
    const [gpodderOnlyPodcasts, setGPodderOnlyPodcasts] = useState<components["schemas"]["GPodderAvailablePodcasts"][]>([])
    const {t} = useTranslation()


    useEffect(() => {
        client.GET("/api/v1/podcasts/available/gpodder").then((gpitem)=>{
           setGPodderOnlyPodcasts(gpitem.data!);
        })
    }, []);


    const addPodcast = (feedUrl: string)=>{
        setGPodderOnlyPodcasts(gpodderOnlyPodcasts.filter(p=>p.podcast!=feedUrl))
        client.POST(  "/api/v1/podcasts/feed", {
            body: {
                rssFeedUrl: feedUrl
            }
        }).then((v) => {
            handleAddPodcast(v.response.status, v.data!.name, t)
        })
    }


    return <table className="text-left text-sm text-stone-900 w-full overflow-y-auto text-[--fg-color]">
        <thead>
        <tr className="border-b border-stone-300">
            <th scope="col" className="pr-2 py-3 text-[--fg-color]">
                #
            </th>
            <th scope="col" className="px-2 py-3 text-[--fg-color]">
                {t('device')}
            </th>
            <th scope="col" className="px-2 py-3 text-[--fg-color]">
                {t('podcasts')}
            </th>
            <th scope="col" className="px-2 py-3 text-[--fg-color]">
                {t('actions')}
            </th>
        </tr>
        </thead>
        <tbody className="">
        {
            gpodderOnlyPodcasts?.map((int, index)=>{
                return <tr key={index}>
                    <td className="px-2 py-4 text-[--fg-color]">{index}</td>
                    <td className="px-2 py-4  text-[--fg-color]">{int.device}</td>
                    <td className="px-2 py-4  text-[--fg-color]">{int.podcast}</td>
                    <td><CustomButtonPrimary onClick={()=>addPodcast(int.podcast)}>{t('add')}</CustomButtonPrimary></td>
                </tr>
                }
            )
        }
        </tbody>
    </table>
}
