import {FC} from "react";
import {PodcastEpisode} from "../store/CommonSlice";

export type IconProps = {
    className?: string,
    onClick?: () => void,
    podcast?: PodcastEpisode
}

export const PlayIcon:FC<IconProps> = ({className, onClick}) => {
    return             <div className={`relative rounded-full ui-bg-foreground w-10 h-10 grid place-items-center active:scale-75 ${className}`} onClick={()=>{
        if (onClick) {
            onClick()
        }
    }
    }>
            <i className="fa-solid text-2xl fa-play text-white ml-1"></i>
    </div>
}
