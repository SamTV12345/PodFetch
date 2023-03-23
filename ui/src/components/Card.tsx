import {createRef, FC, useRef} from "react";
import {Podcast, setPodcasts, updateLikePodcast} from "../store/CommonSlice";
import {Link} from "react-router-dom";
import axios from "axios";
import {apiURL} from "../utils/Utilities";
import {useAppDispatch} from "../store/hooks";

type CardProps = {
    podcast: Podcast
}


export const Card:FC<CardProps> = ({podcast})=>{
    const likeButton = createRef<HTMLElement>()
    const dispatch = useAppDispatch()
    const likePodcast = () => {
        axios.put(apiURL+"/podcast/favored", {
            id: podcast.id,
            favored: !podcast.favored
        })
    }

    return <div className="max-w-sm bg-white border border-gray-200 rounded-lg shadow dark:bg-gray-800 dark:border-gray-700">
        <Link to={"/podcasts/"+podcast.id}>
            <img className="rounded-t-lg" src={podcast.image_url} alt=""/>
        </Link>
        <div className="grid grid-cols-[1fr_auto] p-5">
                <h5 className="mb-2 text-2xl font-bold tracking-tight text-gray-900 dark:text-white">{podcast.name}</h5>
            <i ref={likeButton} className={`fa-star fa-solid text-3xl ${podcast.favored?'text-amber-400': 'text-gray-500'}`} onClick={()=>{
                likeButton.current?.classList.toggle('text-amber-400')
                likePodcast()
                dispatch(updateLikePodcast(podcast.id))
            }}></i>
        </div>
    </div>
}
