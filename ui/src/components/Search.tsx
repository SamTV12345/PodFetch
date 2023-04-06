import {createPortal} from "react-dom";
import axios, {AxiosResponse} from "axios";
import {apiURL, formatTime, removeHTML} from "../utils/Utilities";
import {useState} from "react";
import {useDebounce} from "../utils/useDebounce";
import {useCtrlPressed, useKeyDown} from "../hooks/useKeyDown";
import {PodcastEpisode} from "../store/CommonSlice";
import {useNavigate} from "react-router-dom";
import {Spinner} from "./Spinner";

export const Search = () => {
    const [searchName, setSearchName] = useState<string>('')
    const [open, setOpen] = useState<boolean>(false)
    const [podcastEpisode, setPodcastEpisode] = useState<PodcastEpisode[]>([])
    const navigate = useNavigate()
    const [searching, setSearching] = useState<boolean>()

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

    useCtrlPressed(()=>{
         setOpen(!open)
         document.getElementById('search-input')!.focus()
    }, ["f"])

    useKeyDown(()=>{
        setOpen(false)
    },['Escape'])

    useDebounce(performSearch, 500, [searchName])

    return createPortal(
        <div id="defaultModal" tabIndex={-1} aria-hidden="true" onClick={()=>setOpen(false)}
             className={`overflow-y-auto overflow-x-hidden fixed top-0 right-0 left-0 z-50 md:inset-0 h-modal md:h-full
             ${!open&&'pointer-events-none'}
              z-40 ${open?'opacity-100':'opacity-0'}`}>
            <div className="grid place-items-center h-screen ">
                <div className={`bg-gray-800 max-w-7xl ${open?'opacity-100':'opacity-0'}`} onClick={e=>e.stopPropagation()}>
                    <div className="p-2 rounded-t-lg">
                        <input type="text" className="bg-gray-700 text-white w-full p-2 rounded-lg" value={searchName}
                               onChange={(v)=>setSearchName(v.target.value)} id="search-input"/>
                    </div>
                    <div className="overflow-auto max-h-72 searchfield">
                    {
                        podcastEpisode.length>0&& <hr className="h-px bg-gray-200 border-0 dark:bg-gray-700"/>
                    }
                        { searching?<div className="grid place-items-center"><Spinner className="w-12 h-12"/></div>:
                        podcastEpisode.map((v, i)=>{
                            return <div className="p-2 " key={i}>
                                <div className="flex gap-2">
                                    <img src={v.image_url} alt={v.name} className="w-12 h-12 cursor-pointer" onClick={()=>{
                                        navigate('podcasts/'+v.podcast_id+'/episodes/'+v.id)
                                    }}/>
                                    <div className="flex flex-col">
                                        <div className="text-white font-bold">{v.name}
                                            <span className="text-gray-400 text-sm font-normal"> - {formatTime(v.date_of_recording)}</span>
                                        </div>
                                        <div className="text-gray-400 text-sm font-normal" dangerouslySetInnerHTML={removeHTML(v.description)}></div>
                                    </div>
                                </div>
                            </div>
                        })
                    }
                    </div>
                </div>
            </div>
        </div>, document.getElementById('modal')!)
}
