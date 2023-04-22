import {createPortal} from "react-dom";
import {apiURL, removeHTML} from "../utils/Utilities";
import {useAppDispatch, useAppSelector} from "../store/hooks";
import {setInfoModalDownloaded, setInfoModalPodcastOpen} from "../store/CommonSlice";
import axios from "axios";
import {useTranslation} from "react-i18next";

export const PodcastInfoModal = () => {
    const dispatch = useAppDispatch()
    const selectedPodcastEpisode = useAppSelector(state=>state.common.infoModalPodcast)
    const infoModalOpen = useAppSelector(state=>state.common.infoModalPodcastOpen)
    const {t} =  useTranslation()

    const download = (url: string, filename: string) => {
        const element = document.createElement('a');
        element.setAttribute('href', url);
        element.setAttribute('download', filename);
        element.setAttribute("target", "_blank")
        element.style.display = 'none';
        document.body.appendChild(element);
        element.click();
        document.body.removeChild(element);
    }

    return createPortal( <div id="defaultModal" tabIndex={-1} aria-hidden="true" onClick={()=>dispatch(setInfoModalPodcastOpen(false))}
                              className={`overflow-y-auto overflow-x-hidden fixed top-0 right-0 left-0 z-50 md:inset-0 h-modal md:h-full
             ${!infoModalOpen&&'pointer-events-none'}
              z-40 ${infoModalOpen?'opacity-100':'opacity-0'}`}>
        <div className="grid place-items-center h-screen ">
            <div className={`bg-gray-800 max-w-5xl ${infoModalOpen?'opacity-100':'opacity-0'}`} onClick={e=>e.stopPropagation()}>
                <div className="flex items-start justify-between p-4 border-b rounded-t border-gray-600">
                    <h3 className="text-xl font-semibold text-white">
                        {selectedPodcastEpisode?.name}
                    </h3>
                    <button type="button" onClick={()=>dispatch(setInfoModalPodcastOpen(false))}
                            className="text-gray-400 bg-transparent rounded-lg text-sm p-1.5 ml-auto inline-flex items-center hover:bg-gray-600 hover:text-white"
                            data-modal-hide="defaultModal">
                        <svg aria-hidden="true" className="w-5 h-5" fill="currentColor" viewBox="0 0 20 20"
                             xmlns="http://www.w3.org/2000/svg">
                            <path fillRule="evenodd"
                                  d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z"
                                  clipRule="evenodd"></path>
                        </svg>
                        <span className="sr-only">Close modal</span>
                    </button>
                </div>
                <div className="p-6 space-y-6">
                    {selectedPodcastEpisode&&<p className="text-base leading-relaxed text-gray-400" dangerouslySetInnerHTML={removeHTML(selectedPodcastEpisode.description)}>

                    </p>}
                    <div className="flex gap-4">
                        <button disabled={!selectedPodcastEpisode} className="bg-blue-500 p-1 rounded disabled:bg-blue-900 hover:bg-blue-400" onClick={()=>{
                            if(selectedPodcastEpisode) {
                                download(selectedPodcastEpisode.local_url, selectedPodcastEpisode?.name)
                            }
                        }}>{t('download-computer')}</button>
                        <button disabled={selectedPodcastEpisode&&selectedPodcastEpisode.status==='D'}
                                className="bg-blue-500 disabled:bg-blue-900 p-1 rounded hover:bg-blue-400" onClick={()=>{
                                    if(selectedPodcastEpisode) {
                                        axios.put(apiURL + "/podcast/" + selectedPodcastEpisode?.episode_id + "/episodes/download")
                                            .then(()=>{
                                                dispatch(setInfoModalDownloaded(selectedPodcastEpisode.episode_id))
                                            })

                                    }
                        }}><i className="fa-solid fa-save text-white text-2xl p-2" title={t('download-to-server') as string}></i></button>
                    </div>
                </div>
            </div>
        </div>
    </div>, document.getElementById('modal1')!)

}
