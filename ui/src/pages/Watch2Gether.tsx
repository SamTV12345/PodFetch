import {useEffect, useState} from "react";
import {PodFlix} from "../models/PodFlix";
import axios, {AxiosResponse} from "axios";
import {useTranslation} from "react-i18next";
import {CustomButtonPrimary} from "../components/CustomButtonPrimary";
import {PlaylistDto} from "../models/Playlist";
import useModal from "../store/ModalSlice";
import {Watch2GetherEditModal} from "../components/Watch2GetherEditModal";
import {useWatchTogether} from "../store/Watch2Gether";

export const Watch2Gether = ()=>{
    const [podflixes, setPodflixes] = useState<PodFlix[]>([]);
    const {t} = useTranslation()
    const setModalOpen = useModal(state => state.setOpenModal)
    const setCurrentWatchTogether = useWatchTogether(state => state.setWatchTogetherCreate)

    useEffect(() => {
        axios.get(
            "/watch-together"
        )
            .then((r: AxiosResponse<PodFlix[]>)=>{
                setPodflixes(r.data)
            })
    }, []);




    return <div className={`scrollbox-x  p-2`}>
        <Watch2GetherEditModal/>
        <CustomButtonPrimary className="flex items-center xs:float-right mb-4 xs:mb-10" onClick={()=>{
            setCurrentWatchTogether({
                roomName: "",
            })
            setModalOpen(true)
        }}>
            <span className="material-symbols-outlined leading-[0.875rem]">add</span> {t('add-new')}
        </CustomButtonPrimary>

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
                    {t('admin')}
                </th>
                <th scope="col" className="px-2 py-3 text-[--fg-color]">
                    {t('actions')}
                </th>
            </tr>
            </thead>
            <tbody className="">
            {podflixes?.map((item, index) => {
                return <tr className="">
                    <td className="text-[--fg-color] p-2">
                        {index}
                    </td>
                    <td className="text-[--fg-color]">
                        {item.roomId}
                    </td>
                    <td  className="text-[--fg-color]">
                        {item.admin}
                    </td>
                    <td  className="text-[--fg-color]">
                        {item.roomName}
                    </td>
                </tr>
            })}
            </tbody>
        </table>
    </div>
}
