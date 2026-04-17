import { FC, useState } from 'react'
import * as Dialog from '@radix-ui/react-dialog'
import { useTranslation } from 'react-i18next'
import { removeHTML } from '../utils/Utilities'
import { Heading2 } from './Heading2'
import 'material-symbols/outlined.css'
import { $api } from '../utils/http'
import { PodcastEpisodeChapterTable } from './PodcastEpisodeChapterTable'
import { components } from '../../schema'

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

type PodcastInfoModalProps = {
    open: boolean
    onOpenChange: (open: boolean) => void
    episode: components["schemas"]["PodcastEpisodeDto"] | undefined
}

export const PodcastInfoModal: FC<PodcastInfoModalProps> = ({ open, onOpenChange, episode }) => {
    const { t } = useTranslation()
    const [selectedTab, setSelectedTab] = useState<'description' | 'chapters'>('description')
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
            onOpenChange(false)
        })
    }

    return (
        <Dialog.Root open={open} onOpenChange={onOpenChange}>
            <Dialog.Portal>
                <Dialog.Overlay className="fixed inset-0 bg-[rgba(0,0,0,0.5)] backdrop-blur-sm z-30" />
                <Dialog.Content className="fixed inset-0 z-40 flex items-center justify-center p-4">
                    <div className="relative ui-surface w-full max-w-2xl max-h-[calc(100vh-2rem)] overflow-y-auto p-8 rounded-2xl shadow-[0_4px_16px_rgba(0,0,0,var(--shadow-opacity))]">
                        <Dialog.Close className="absolute top-4 right-4 bg-transparent">
                            <span className="material-symbols-outlined ui-modal-close hover:ui-modal-close-hover">close</span>
                            <span className="sr-only">{t('closeModal')}</span>
                        </Dialog.Close>
                        <div className="mb-4">
                            <Dialog.Title asChild>
                                <Heading2 className="inline align-middle mr-2 break-all">{episode?.name || ''}</Heading2>
                            </Dialog.Title>
                            <Dialog.Description className="sr-only">
                                {t('podcast-episode-details')}
                            </Dialog.Description>
                            <span
                                className={`material-symbols-outlined align-middle ${episode ? 'cursor-pointer ui-icon hover:ui-icon-hover' : 'text-stone-300'}`}
                                title={t('download-computer') as string}
                                onClick={() => {
                                    if (episode) {
                                        const extension = inferExtension(episode.local_url || episode.url)
                                        const downloadUrl = episode.status ? episode.local_url : episode.url
                                        download(downloadUrl, `${episode.name}.${extension}`)
                                    }
                                }}
                            >save</span>
                            {episode?.status &&
                                <span
                                    onClick={() => deleteEpisodeDownloadOnServer(episode?.episode_id)}
                                    className="material-symbols-outlined align-middle cursor-pointer ui-text-danger hover:ui-text-danger-hover"
                                    title={t('delete') as string}
                                >delete</span>
                            }
                        </div>
                        <ul className="flex flex-wrap gap-2 border-b ui-border mb-6 ui-text-muted">
                            <li
                                onClick={() => setSelectedTab('description')}
                                className={`cursor-pointer inline-block px-2 py-4 ${selectedTab === 'description' && 'border-b-2 ui-border-accent ui-text-accent'}`}
                            >
                                {t('description')}
                            </li>
                            <li
                                onClick={() => setSelectedTab('chapters')}
                                className={`cursor-pointer inline-block px-2 py-4 ${selectedTab === 'chapters' && 'border-b-2 ui-border-accent ui-text-accent'}`}
                            >
                                {t('chapters')}
                            </li>
                        </ul>
                        {episode &&
                            <>
                                {selectedTab === 'description' ? (
                                    <p className="leading-[1.75] text-sm ui-text" dangerouslySetInnerHTML={removeHTML(episode.description)} />
                                ) : (
                                    <PodcastEpisodeChapterTable podcastEpisode={episode} className="overflow-auto max-h-1/2" />
                                )}
                            </>
                        }
                    </div>
                </Dialog.Content>
            </Dialog.Portal>
        </Dialog.Root>
    )
}
