import {createPortal} from "react-dom";
import {setDetailedAudioPlayerOpen} from "../store/CommonSlice";
import {useAppDispatch, useAppSelector} from "../store/hooks";
import {removeHTML} from "../utils/Utilities";
import ProgressBar from "./AudioProgressBar";
import {MenuBarPlayer} from "./MenuBarPlayer";
import {FC, RefObject} from "react";
import {VolumeSlider} from "./VolumeSlider";
import {AudioAmplifier} from "../models/AudioAmplifier";

type DetailedAudioPlayerProps = {
    refItem: RefObject<HTMLAudioElement>,
    audioAmplifier: AudioAmplifier|undefined,
    setAudioAmplifier: (audioAmplifier: AudioAmplifier|undefined)=>void
}
export const DetailedAudioPlayer:FC<DetailedAudioPlayerProps> = ({refItem, audioAmplifier}) => {
    const dispatch = useAppDispatch()
    const detailedAudioPlayerOpen = useAppSelector(state => state.common.detailedAudioPlayerOpen)
    const selectedPodcast = useAppSelector(state => state.audioPlayer.currentPodcastEpisode)

    return createPortal(<div id="defaultModal" tabIndex={-1} aria-hidden="true" onClick={()=>dispatch(setDetailedAudioPlayerOpen(false))}
                                           className={`overflow-y-auto overflow-x-hidden fixed top-0 right-0 left-0 z-60 md:inset-0 h-modal md:h-full
             ${!detailedAudioPlayerOpen&&'pointer-events-none'}
              z-40 ${detailedAudioPlayerOpen?'opacity-100':'animate-opacity'} duration-75`}>
        <div className="bg-gray-800 h-full grid grid-rows-[1fr_auto]"  onClick={event => event.stopPropagation()}>
            <div className="grid grid-cols-[1fr_2fr]">
                <div className="grid place-items-center">
                    <img src={selectedPodcast?.local_image_url} alt={selectedPodcast?.name} className="h-80 object-cover shadow-lg shadow-amber-600"/>
                </div>
                <div className="grid place-items-center text-white text-2xl">
                    <div className="max-h-80 overflow-y-auto">{selectedPodcast?.description&& removeHTML(selectedPodcast.description)}</div>
                </div>
            </div>
            <div className="col-span-2 mb-3">
                <ProgressBar audioplayerRef={refItem} className={"text-white"}/>
                <div className="grid-cols-3 grid">
                    <div></div>
                    <MenuBarPlayer refItem={refItem}/>
                    <VolumeSlider refItem={refItem} audioAmplifier={audioAmplifier}/>
                </div>
            </div>
        </div>
        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor" className=" ml-2 w-8 h-8 absolute top-0 left-0 text-white">
            <path strokeLinecap="round" strokeLinejoin="round" d="M19.5 8.25l-7.5 7.5-7.5-7.5" />
        </svg>

    </div>, document.getElementById('modal')!)
}
