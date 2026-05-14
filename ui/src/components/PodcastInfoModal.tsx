import { FC, useState } from 'react'
import { Dialog, DialogContent, DialogDescription, DialogTitle } from '@/components/ui/dialog'
import { useTranslation } from 'react-i18next'
import { removeHTML } from '../utils/Utilities'
import { Heading2 } from './Heading2'
import { Download, Trash2 } from 'lucide-react'
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
        <Dialog open={open} onOpenChange={onOpenChange}>
            {/*
              Layout note: shadcn's DialogContent defaults to `grid`; for a
              scrolling tab body we need a flex column with a real height
              budget. `max-h-[85vh]` gives the chapter table somewhere to
              live, and `min-h-0` on the flex-1 child lets it shrink below
              its content (so overflow-y-auto actually engages instead of
              the dialog growing past the viewport).
            */}
            <DialogContent className="w-full max-w-2xl sm:max-w-2xl max-h-[85vh] flex flex-col">
                <div className="mb-4 shrink-0">
                    <DialogTitle render={<Heading2 className="inline align-middle mr-2 break-all">{episode?.name || ''}</Heading2>} />
                    <DialogDescription className="sr-only">
                        {t('podcast-episode-details')}
                    </DialogDescription>
                    <Download
                        size={20}
                        className={`inline-block align-middle ${episode ? 'cursor-pointer ui-icon hover:ui-icon-hover' : 'ui-text-disabled'}`}
                        aria-label={t('download-computer') as string}
                        onClick={() => {
                            if (episode) {
                                const extension = inferExtension(episode.local_url || episode.url)
                                const downloadUrl = episode.status ? episode.local_url : episode.url
                                download(downloadUrl, `${episode.name}.${extension}`)
                            }
                        }}
                    />
                    {episode?.status &&
                        <Trash2
                            size={20}
                            onClick={() => deleteEpisodeDownloadOnServer(episode?.episode_id)}
                            className="inline-block align-middle ml-2 cursor-pointer ui-text-danger hover:ui-text-danger-hover"
                            aria-label={t('delete') as string}
                        />
                    }
                </div>
                <ul className="flex flex-wrap gap-2 border-b ui-border mb-4 ui-text-muted shrink-0">
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
                    <div className="flex-1 min-h-0 overflow-y-auto pr-1">
                        {selectedTab === 'description' ? (
                            <p className="leading-[1.75] text-sm ui-text" dangerouslySetInnerHTML={removeHTML(episode.description)} />
                        ) : (
                            <PodcastEpisodeChapterTable podcastEpisode={episode} />
                        )}
                    </div>
                }
            </DialogContent>
        </Dialog>
    )
}
