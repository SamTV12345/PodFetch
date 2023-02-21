import {PlayIcon} from "./PlayIcon";
import {createRef, useRef} from "react";
import ProgressBar from "./AudioProgressBar";

export const AudioPlayer = () => {
    const ref = createRef<HTMLAudioElement>()

    return <div className="sticky bottom-0 w-full bg-gray-800 h-12">
        <ProgressBar/>
        <div className="place-items-center grid">
                <div className="flex gap-3 align-baseline">
                <i className="fa-solid fa-backward text-xl text-white"></i>
            <PlayIcon className="text-white text-xl align-top"/>
                <i className="fa-solid fa-forward h-6 text-xl text-white"></i>
            </div>
        </div>
        <audio ref={ref}></audio>
    </div>
}
