import { useState } from 'react'
import * as Dialog from '@radix-ui/react-dialog'
import { useCtrlPressed } from '../hooks/useKeyDown'
import { EpisodeSearch } from './EpisodeSearch'
import { useNavigate } from 'react-router-dom'

export const EpisodeSearchModal = () => {
    const [open, setOpen] = useState<boolean>(false)
    const navigate = useNavigate()

    useCtrlPressed(() => {
        setOpen(prev => {
            const next = !prev
            if (!next) {
                document.getElementById('search-input')?.blur()
            } else {
                setTimeout(() => document.getElementById('search-input')?.focus(), 0)
            }
            return next
        })
    }, ['f'])

    const handleOpenChange = (nextOpen: boolean) => {
        setOpen(nextOpen)
        if (!nextOpen) {
            document.getElementById('search-input')?.blur()
        }
    }

    return (
        <Dialog.Root open={open} onOpenChange={handleOpenChange}>
            <Dialog.Portal>
                <Dialog.Overlay className="fixed inset-0 bg-[rgba(0,0,0,0.5)] backdrop-blur-sm z-30" />
                <Dialog.Content className="fixed inset-0 z-40 flex items-center justify-center p-4" aria-label="Episode search">
                    <Dialog.Title className="sr-only">Episode search</Dialog.Title>
                    <div className="ui-surface max-h-screen max-w-4xl p-4 rounded-2xl shadow-[0_4px_16px_rgba(0,0,0,var(--shadow-opacity))] w-full">
                        {/*
                        Max-height of search results section should be the lesser of:
                            - 24rem
                            - Or, for when screen height is smaller: viewport height - vertical padding/spacing - height of search field
                        */}
                        <EpisodeSearch onClickResult={(episode) => {
                            setOpen(false)
                            navigate(`/podcasts/${episode.podcast_id}/episodes/${episode.id}`)
                        }} classNameResults="max-h-[min(24rem,calc(100vh-3rem-3rem))]" showBlankState={false} />
                    </div>
                </Dialog.Content>
            </Dialog.Portal>
        </Dialog.Root>
    )
}
