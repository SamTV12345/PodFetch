import {createRef, FC} from "react"
import {Link} from "react-router-dom"
import axios from "axios"
import {apiURL} from "../utils/Utilities"
import {useAppDispatch} from "../store/hooks"
import {Podcast, updateLikePodcast} from "../store/CommonSlice"
import "material-symbols/outlined.css"

type CardProps = {
    podcast: Podcast
}

export const Card:FC<CardProps> = ({podcast})=>{
    const likeButton = createRef<HTMLElement>()
    const dispatch = useAppDispatch()
    const likePodcast = () => {
        axios.put(apiURL+"/podcast/favored", {
            id: podcast.id,
            favored: !podcast.favorites
        })
    }

    return <Link className="group" to={podcast.id+"/episodes"}>
        <div className="relative mb-2">
            <img className={`rounded-xl transition-shadow group-hover:shadow-[0_4px_32px_rgba(0,0,0,0.3)] ${!podcast.active?'opacity-20':''}`} src={podcast.image_url} alt=""/>

            <span ref={likeButton} className={`material-symbols-outlined filled absolute top-2 right-2 h-6 w-6 filled ${podcast.favorites?'text-rose-700 hover:text-rose-600': 'text-stone-200 hover:text-stone-100'}`} onClick={(e)=>{
                // Prevent icon click from triggering link to podcast detail
                e.preventDefault()

                likeButton.current?.classList.toggle('fill-amber-400')
                likePodcast()
                dispatch(updateLikePodcast(podcast.id))
            }}>favorite</span>
        </div>

        <div>
            <span className="block font-bold leading-[1.2] mb-2 text-stone-900 transition-color group-hover:text-stone-600">{podcast.name}</span>
            <span className="block leading-[1.2] text-sm text-stone-500">{podcast.author}</span>
        </div>
    </Link>
}
