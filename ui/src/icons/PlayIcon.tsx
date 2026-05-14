import {FC} from "react";
import {Play} from "lucide-react";
import {PodcastEpisode} from "../store/CommonSlice";

export type IconProps = {
    className?: string,
    onClick?: () => void,
    podcast?: PodcastEpisode
}

export const PlayIcon:FC<IconProps> = ({className, onClick}) => {
    return <div
        className={`relative rounded-full ui-bg-foreground w-10 h-10 grid place-items-center active:scale-75 ${className ?? ''}`}
        onClick={() => onClick?.()}
    >
        <Play size={20} className="text-white ml-0.5" fill="currentColor"/>
    </div>
}
