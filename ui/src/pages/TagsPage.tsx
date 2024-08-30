import {Heading1} from "../components/Heading1";
import {useTranslation} from "react-i18next";
import useCommon from "../store/CommonSlice";
import {useEffect} from "react";
import axios, {AxiosResponse} from "axios";
import {PodcastTags} from "../models/PodcastTags";
import {PlaylistDto} from "../models/Playlist";
import {CustomInput} from "../components/CustomInput";

export const TagsPage = ()=>{
    const {t}  = useTranslation()
    const tags = useCommon(state=>state.tags)
    const setTags = useCommon(state=>state.setPodcastTags)

    useEffect(() => {
        if (tags.length === 0) {
            axios.get('/tags').then((response: AxiosResponse<PodcastTags[]>) => {
                setTags(response.data)
            })
        }
    }, []);

    return (
        <>
        <Heading1>{t('tag_other')}</Heading1>
            <table className="text-left text-sm text-[--fg-color]">
                <thead>
                <tr className="border-b border-stone-300">
                    <th scope="col" className="px-2 py-3 text-[--fg-color]">
                        {t('tag_one')}
                    </th>
                    <th scope="col" className="px-2 py-3 text-[--fg-color]">
                        {t('actions')}
                    </th>
                </tr>
                </thead>
                <tbody>
                {
                    tags.map(tag=> {
                        return <tr className="border-b border-stone-300 " key={tag.id}>
                            <td className="px-2 py-4 flex items-center text-[--fg-color]">
                                <CustomInput onBlur={()=>{
                                    axios.put(`/tags/${tag.id}`, {
                                        name: tag.name,
                                        color: tag.color
                                    })
                                }} value={tag.name} onChange={(event)=>{
                                    setTags(tags.map(t=>{
                                        if (t.id === tag.id) {
                                            return {
                                                ...t,
                                                name: event.target.value
                                            }
                                        }
                                        return t
                                    }))
                                }}/>
                            </td>
                            <td>
                                <button className="px-2 py-1 text-[--fg-color] rounded-md bg-red-700" onClick={() => {
                                    console.log(tags, tag)
                                    axios.delete(`/tags/${tag.id}`).then(() => {
                                       setTags(tags.filter(tagfiltered => tagfiltered.id !== tag.id))
                                    })
                                }}>{t('delete')}</button>
                            </td>
                        </tr>
                    })
                }
                </tbody>
            </table>
        </>
    )
}
