import { createPortal } from 'react-dom'
import { useTranslation } from 'react-i18next'
import axios from 'axios'
import {  removeHTML } from '../utils/Utilities'
import useCommon from '../store/CommonSlice'
import { Heading2 } from './Heading2'
import 'material-symbols/outlined.css'

export const PodcastInfoModal = () => {
    const infoModalOpen = useCommon(state => state.infoModalPodcastOpen)
    const selectedPodcastEpisode = useCommon(state => state.infoModalPodcast)
    const { t } =  useTranslation()
    const setInfoModalPodcastOpen = useCommon(state => state.setInfoModalPodcastOpen)
    const download = (url: string, filename: string) => {
        const element = document.createElement('a')
        element.setAttribute('href', url)
        element.setAttribute('download', filename)
        element.setAttribute('target', '_blank')
        element.style.display = 'none'
        document.body.appendChild(element)
        element.click()
        document.body.removeChild(element)
    }

    const deleteEpisodeDownloadOnServer = (episodeId: string) => {
        axios.delete(  '/episodes/' + episodeId + '/download').then(() => {
            setInfoModalPodcastOpen(false)
        })
    }

    return createPortal(
        <div
            id="defaultModal"
            tabIndex={-1}
            aria-hidden="true"
            onClick={() => setInfoModalPodcastOpen(false)}
            className={`fixed inset-0 grid place-items-center bg-[rgba(0,0,0,0.5)] backdrop-blur overflow-y-auto overflow-x-hidden transition-opacity z-30
            ${!infoModalOpen && 'pointer-events-none'}
            ${infoModalOpen ? 'opacity-100' : 'opacity-0'}`}
        >
            <div className={`relative bg-[--bg-color] max-w-2xl p-8 rounded-2xl shadow-[0_4px_16px_rgba(0,0,0,var(--shadow-opacity))] ${infoModalOpen ? 'opacity-100' : 'opacity-0'}`} onClick={e => e.stopPropagation()}>
                <button
                    type="button"
                    onClick={() => setInfoModalPodcastOpen(false)}
                    className="absolute top-4 right-4 bg-transparent"
                    data-modal-hide="defaultModal"
                >
                    <span className="material-symbols-outlined text-[--modal-close-color] hover:text-[--modal-close-color-hover]">close</span>
                    <span className="sr-only">Close modal</span>
                </button>

                <div className="mb-4">
                    <Heading2 className="inline align-middle mr-2">{selectedPodcastEpisode?.name || ''}</Heading2>

                    {/* Save icon */}
                    <span className={`material-symbols-outlined align-middle ${selectedPodcastEpisode ? 'cursor-pointer text-[--fg-icon-color] hover:text-[--fg-icon-color-hover]' : 'text-stone-300'}`} title={t('download-computer') as string} onClick={() => {
                        if (selectedPodcastEpisode) {
                            download(selectedPodcastEpisode.local_url, selectedPodcastEpisode.name + ".mp3")
                        }
                    }}>save</span>

                    {/* Delete icon */}
                    {selectedPodcastEpisode?.status &&
                        <span onClick={() => deleteEpisodeDownloadOnServer(selectedPodcastEpisode?.episode_id)} className="material-symbols-outlined align-middle cursor-pointer text-[--danger-fg-color] hover:text-[--danger-fg-color-hover]" title={t('delete') as string}>delete</span>
                    }
                </div>

                {selectedPodcastEpisode &&
                    <p className="leading-[1.75] text-sm text-[--fg-color]" dangerouslySetInnerHTML={removeHTML(selectedPodcastEpisode.description)}>
                    </p>
                }
            </div>
        </div>, document.getElementById('modal1')!
    )
}
