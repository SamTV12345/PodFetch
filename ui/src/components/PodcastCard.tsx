import { createRef, FC } from 'react'
import { Link } from 'react-router-dom'
import axios from 'axios'
import {prependAPIKeyOnAuthEnabled} from '../utils/Utilities'
import useCommon, { Podcast } from '../store/CommonSlice'
import 'material-symbols/outlined.css'

type PodcastCardProps = {
    podcast: Podcast
}

export const PodcastCard: FC<PodcastCardProps> = ({ podcast }) => {
    const likeButton = createRef<HTMLElement>()
    const updateLikePodcast = useCommon(state => state.updateLikePodcast)

    const likePodcast = () => {
        axios.put( '/podcast/favored', {
            id: podcast.id,
            favored: !podcast.favorites
        })
    }

    return (
        <Link className="group" to={podcast.id + '/episodes'}>
            <div className="relative mb-2">
                <img className={`rounded-xl transition-shadow group-hover:shadow-[0_4px_32px_rgba(0,0,0,var(--shadow-opacity))] ${!podcast.active ? 'opacity-20' : ''}`} src={prependAPIKeyOnAuthEnabled(podcast.image_url)} alt=""/>

                <span ref={likeButton} className={`material-symbols-outlined filled absolute top-2 right-2 h-6 w-6 filled ${podcast.favorites?'text-rose-700 hover:text-rose-600': 'text-stone-200 hover:text-stone-100'}`} onClick={(e) => {
                    // Prevent icon click from triggering link to podcast detail
                    e.preventDefault()

                    likeButton.current?.classList.toggle('fill-amber-400')
                    likePodcast()
                    updateLikePodcast(podcast.id)
                }}>favorite</span>
            </div>

            <div>
                <span className="block font-bold leading-[1.2] mb-2 text-[--fg-color] transition-colors group-hover:text-[--fg-color-hover]">{podcast.name}</span>
                <span className="block leading-[1.2] text-sm text-[--fg-secondary-color]">{podcast.author}</span>
            </div>
        </Link>
    )
}
