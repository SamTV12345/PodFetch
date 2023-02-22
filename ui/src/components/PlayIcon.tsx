import {FC} from "react";
import {PodcastEpisode} from "../store/CommonSlice";

export type IconProps = {
    className?: string,
    onClick?: () => void,
    podcast?: PodcastEpisode
}

export const PlayIcon:FC<IconProps> = ({className, onClick, podcast}) => {
    return   <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth="1.5" onClick={()=>{
        if (onClick) {
            onClick()
        }
        }
    }
                                            stroke="currentColor" className={`w-6 h-6 cursor-pointer ${className}`}>
        <path strokeLinecap="round" strokeLinejoin="round"
              d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
        <path strokeLinecap="round" strokeLinejoin="round"
              d="M15.91 11.672a.375.375 0 010 .656l-5.603 3.113a.375.375 0 01-.557-.328V8.887c0-.286.307-.466.557-.327l5.603 3.112z"/>
    </svg>
}
