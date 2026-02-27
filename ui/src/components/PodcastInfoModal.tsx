import { createPortal } from 'react-dom'
import { useTranslation } from 'react-i18next'
import {  removeHTML } from '../utils/Utilities'
import useCommon from '../store/CommonSlice'
import { Heading2 } from './Heading2'
import 'material-symbols/outlined.css'
import {$api} from "../utils/http";
import {useState} from "react";
import {PodcastEpisodeChapterTable} from "./PodcastEpisodeChapterTable";

const inferExtension = (url: string): string => {
    try {
        const cleanPath = url.split('?')[0] || '';
        const parts = cleanPath.split('.');
        const ext = parts[parts.length - 1]?.toLowerCase() || '';
        return /^[a-z0-9]{2,8}$/.test(ext) ? ext : 'mp3';
    } catch {
        return 'mp3';
    }
}

export const PodcastInfoModal = () => {
    const infoModalOpen = useCommon(state => state.infoModalPodcastOpen)
    const selectedPodcastEpisode = useCommon(state => state.infoModalPodcast)
    const { t } =  useTranslation()
    const [selectedTab, setSelectedTab] = useState<'description'|'chapters'>('description')
    const setInfoModalPodcastOpen = useCommon(state => state.setInfoModalPodcastOpen)
    const deleteEpisodeDownloadMutation = $api.useMutation('delete', '/api/v1/episodes/{id}/download')
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
        deleteEpisodeDownloadMutation.mutateAsync({
            params: {
                path: {
                    id: episodeId
                }
            }
        }).then(() => {
            setInfoModalPodcastOpen(false)
        })
    }

    return createPortal(
        <div
            id="defaultModal"
            tabIndex={-1}
            aria-hidden="true"
            onClick={() => setInfoModalPodcastOpen(false)}
            className={`fixed inset-0 z-30 bg-[rgba(0,0,0,0.5)] backdrop-blur p-4 flex items-center justify-center transition-opacity
        ${!infoModalOpen && 'pointer-events-none'}
        ${infoModalOpen ? 'opacity-100' : 'opacity-0'}`}
        >
            <div
                className={`relative bg-(--bg-color) w-full max-w-2xl max-h-[calc(100vh-2rem)] overflow-y-auto p-8 rounded-2xl shadow-[0_4px_16px_rgba(0,0,0,var(--shadow-opacity))] ${infoModalOpen ? 'opacity-100' : 'opacity-0'}`}
                onClick={e => e.stopPropagation()}
            >
                <button
                    type="button"
                    onClick={() => setInfoModalPodcastOpen(false)}
                    className="absolute top-4 right-4 bg-transparent"
                    data-modal-hide="defaultModal"
                >
                    <span className="material-symbols-outlined text-(--modal-close-color) hover:text-(--modal-close-color-hover)">close</span>
                    <span className="sr-only">Close modal</span>
                </button>

                <div className="mb-4">
                    <Heading2 className="inline align-middle mr-2 break-all">{selectedPodcastEpisode?.name || ''}</Heading2>

                    <span className={`material-symbols-outlined align-middle ${selectedPodcastEpisode ? 'cursor-pointer text-(--fg-icon-color) hover:text-(--fg-icon-color-hover)' : 'text-stone-300'}`} title={t('download-computer') as string} onClick={() => {
                        if (selectedPodcastEpisode) {
                            const extension = inferExtension(selectedPodcastEpisode.local_url || selectedPodcastEpisode.url)
                            const downloadUrl = selectedPodcastEpisode.status ? selectedPodcastEpisode.local_url : selectedPodcastEpisode.url
                            download(downloadUrl, `${selectedPodcastEpisode.name}.${extension}`)
                        }
                    }}>save</span>

                    {selectedPodcastEpisode?.status &&
                        <span onClick={() => deleteEpisodeDownloadOnServer(selectedPodcastEpisode?.episode_id)} className="material-symbols-outlined align-middle cursor-pointer text-(--danger-fg-color) hover:text-(--danger-fg-color-hover)" title={t('delete') as string}>delete</span>
                    }
                </div>

                <ul className="flex flex-wrap gap-2 border-b border-(--border-color) mb-6 text-(--fg-secondary-color)">
                    <li onClick={()=>setSelectedTab('description')} className={`cursor-pointer inline-block px-2 py-4 ${selectedTab === 'description' && 'border-b-2 border-(--accent-color) text-(--accent-color)'}`}>
                        {t('description')}
                    </li>
                    <li onClick={()=>setSelectedTab('chapters')} className={`cursor-pointer inline-block px-2 py-4 ${selectedTab === 'chapters' && 'border-b-2 border-(--accent-color) text-(--accent-color)'}`}>
                        {t('chapters')}
                    </li>
                </ul>

                {selectedPodcastEpisode &&
                    <>
                        {
                            selectedTab === 'description' ? (
                                <p className="leading-[1.75] text-sm text-(--fg-color)" dangerouslySetInnerHTML={removeHTML(selectedPodcastEpisode.description)}/>
                            ): (<PodcastEpisodeChapterTable podcastEpisode={selectedPodcastEpisode} className="overflow-auto max-h-1/2"/>)
                        }
                    </>
                }
            </div>
        </div>, document.getElementById('modal1')!)
}
