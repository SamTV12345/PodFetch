import {useEffect, useState} from "react";
import {PodFlix} from "../models/PodFlix";
import axios, {AxiosResponse} from "axios";
import {useTranslation} from "react-i18next";

export const Watch2Gether = ()=>{
    const [podflixes, setPodflixes] = useState<PodFlix[]>([]);
    const {t} = useTranslation()
    useEffect(() => {
        axios.get(
            "/watch-together"
        )
            .then((r: AxiosResponse<PodFlix[]>)=>{
                setPodflixes(r.data)
            })
    }, []);


    return <div className={`scrollbox-x  p-2`}>
        <table className="text-left text-sm text-stone-900 w-full overflow-y-auto text-[--fg-color]">
            <thead>
            <tr className="border-b border-stone-300">
                <th scope="col" className="pr-2 py-3 text-[--fg-color]">
                    #
                </th>
                <th scope="col" className="px-2 py-3 text-[--fg-color]">
                    {t('podflix')}
                </th>
                <th scope="col" className="px-2 py-3 text-[--fg-color]">
                    {t('actions')}
                </th>
            </tr>
            </thead>
            <tbody className="">
            {podflixes?.map((item, index) => {
                return <tr className="border-2 border-white">
                    <td className="text-[--fg-color] p-2">
                        {index}
                    </td>
                    <td className="text-[--fg-color]">
                        {item.roomId}
                    </td>
                    <td>
                        {item.admin}
                    </td>
                    <td>
                        {item.roomName}
                    </td>
                </tr>
            })}
            </tbody>
        </table>
    </div>
}
