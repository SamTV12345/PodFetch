import {createPortal} from "react-dom"
import {useTranslation} from "react-i18next"
import {preparePath, removeHTML} from "../utils/Utilities"
import {useAppDispatch, useAppSelector} from "../store/hooks"
import {setInfoModalPodcastOpen} from "../store/CommonSlice"
import {Heading2} from "./Heading2"
import "material-symbols/outlined.css"

export const InfoModal = () => {
    const dispatch = useAppDispatch()
    const infoHeading = useAppSelector(state=>state.common.infoHeading)
    const infoText = useAppSelector(state=>state.common.infoText)
    const infoModalOpen = useAppSelector(state=>state.common.infoModalPodcastOpen)
    const {t} =  useTranslation()

    return createPortal( <div id="defaultModal" tabIndex={-1} aria-hidden="true" onClick={()=>dispatch(setInfoModalPodcastOpen(false))}
                              className={`fixed inset-0 grid place-items-center bg-[rgba(0,0,0,0.5)] backdrop-blur overflow-y-auto overflow-x-hidden transition-opacity z-30
         ${!infoModalOpen&&'pointer-events-none'}
         ${infoModalOpen?'opacity-100':'opacity-0'}`}>

        <div className={`relative bg-white max-w-2xl p-8 rounded-2xl shadow-[0_4px_16px_rgba(0,0,0,0.2)] ${infoModalOpen?'opacity-100':'opacity-0'}`} onClick={e=>e.stopPropagation()}>
            <button type="button" onClick={()=>dispatch(setInfoModalPodcastOpen(false))}
                    className="absolute top-4 right-4 bg-transparent"
                    data-modal-hide="defaultModal">
                <span className="material-symbols-outlined text-stone-400 hover:text-stone-600">close</span>
                <span className="sr-only">Close modal</span>
            </button>

            <div className="mb-4">
                {infoHeading&&<Heading2 className="inline align-middle mr-2">{t(infoHeading!) || ''}</Heading2>}
            </div>

            {infoText&&<p className="leading-[1.75] text-sm text-stone-900">{t(infoText!)}
            </p>}
        </div>
    </div>, document.getElementById('modal1')!)
}
