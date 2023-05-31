import {FC} from "react";
import {PodcastEpisode} from "../store/CommonSlice";

export type IconProps = {
    className?: string,
    onClick?: () => void,
    podcast?: PodcastEpisode
}

export const PlayIcon:FC<IconProps> = ({className, onClick}) => {
    return             <div className={`relative rounded-full bg-black w-10 h-10 grid place-items-center bg-gray-700 hover:bg-gray-600 active:scale-75 ${className}`} onClick={()=>{
        if (onClick) {
            onClick()
        }
    }
    }>
            <i className="fa-solid text-2xl fa-play text-white ml-1"></i>
    </div>
}
