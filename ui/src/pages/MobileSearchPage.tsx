import {Spinner} from "../components/Spinner";
import {apiURL, formatTime, removeHTML} from "../utils/Utilities";
import {useState} from "react";
import {PodcastEpisode} from "../store/CommonSlice";
import {useNavigate} from "react-router-dom";
import {useTranslation} from "react-i18next";
import axios, {AxiosResponse} from "axios";
import {useDebounce} from "../utils/useDebounce";
import {EmptyResultIcon} from "../icons/EmptyResultIcon";

export const MobileSearchPage = ()=>{
    const [podcastEpisode, setPodcastEpisode] = useState<PodcastEpisode[]>([])
    const navigate = useNavigate()
    const [searching, setSearching] = useState<boolean>()
    const [searchName, setSearchName] = useState<string>('')
    const {t} = useTranslation()

    const performSearch = ()=>{
        if(searchName.trim().length>0) {
            setSearching(true)
            axios.get(apiURL + "/podcasts/" + searchName + "/query")
                .then((v: AxiosResponse<PodcastEpisode[]>) => {
                    setPodcastEpisode(v.data)
                    setSearching(false)
                })
        }
    }
    useDebounce(performSearch, 500, [searchName])

    return <div className="p-3">
        <h1 className="font-bold text-2xl">{t('search-podcasts')}</h1>
        <div className="p-2 rounded-t-lg">
        <input type="text" placeholder={t('search-podcasts')!} className="bg-gray-700 text-white w-full p-2 rounded-lg" value={searchName}
               onChange={(v)=>setSearchName(v.target.value)} id="search-input"/>
    </div>
        <div className="">
            {
                podcastEpisode.length>0&& <hr className="h-px border-0 bg-gray-700"/>
            }
            { searching?<Spinner className="w-12 h-12"/>:
                <div>
                    {
                        podcastEpisode.length===0&&<div className="grid place-items-center"><EmptyResultIcon/></div>
                    }
                        <div className="bg-slate-600 m-2">
            {podcastEpisode.map((v, i)=>{
                    return <div className="p-2 " key={i}>
                        <div className="flex gap-2">
                            <img src={v.image_url} alt={v.name} className="w-12 h-12 cursor-pointer" onClick={()=>{
                                navigate('/podcasts/'+v.podcast_id+'/episodes/'+v.id)
                            }}/>
                            <div className="flex flex-col">
                                <div className="text-white font-bold">{v.name}
                                    <span className="text-gray-400 text-sm font-normal"> - {formatTime(v.date_of_recording)}</span>
                                </div>
                                <div className="text-gray-400 text-sm font-normal" dangerouslySetInnerHTML={removeHTML(v.description)}></div>
                            </div>
                        </div>
                    </div>})
            }
                </div>
        </div>
            }
        </div>
    </div>
}
