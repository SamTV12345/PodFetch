import {createPortal} from "react-dom";
import {setDetailedAudioPlayerOpen} from "../store/CommonSlice";
import {useAppDispatch, useAppSelector} from "../store/hooks";

export const DetailedAudioPlayer = () => {
    const dispatch = useAppDispatch()
    const detailedAudioPlayerOpen = useAppSelector(state => state.common.detailedAudioPlayerOpen)
    const selectedPodcast = useAppSelector(state => state.audioPlayer.currentPodcastEpisode)

    return createPortal(<div id="defaultModal" tabIndex={-1} aria-hidden="true" onClick={()=>dispatch(setDetailedAudioPlayerOpen(false))}
                                           className={`overflow-y-auto overflow-x-hidden fixed top-0 right-0 left-0 z-50 md:inset-0 h-modal md:h-full
             ${!detailedAudioPlayerOpen&&'pointer-events-none'}
              z-40 ${detailedAudioPlayerOpen?'opacity-100':'opacity-0'}`}>
        <div className="grid grid-cols-2 bg-gray-800 h-full">
            <div className="grid place-items-center">
                <img src={selectedPodcast?.local_image_url} alt={selectedPodcast?.name} className="h-80 object-cover shadow-lg shadow-amber-600"/>
            </div>
            <div>

            </div>
        </div>
    </div>, document.getElementById('modal')!)
}
