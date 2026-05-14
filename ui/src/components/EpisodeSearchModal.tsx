import { useState } from 'react'
import { Dialog, DialogContent, DialogTitle } from '@/components/ui/dialog'
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
        <Dialog open={open} onOpenChange={handleOpenChange}>
            <DialogContent
                aria-label="Episode search"
                className="max-w-4xl w-full p-4 sm:max-w-4xl"
            >
                <DialogTitle className="sr-only">Episode search</DialogTitle>
                {/*
                Max-height of search results section should be the lesser of:
                    - 24rem
                    - Or, for when screen height is smaller: viewport height - vertical padding/spacing - height of search field
                */}
                <EpisodeSearch onClickResult={(episode) => {
                    setOpen(false)
                    navigate(`/podcasts/${episode.podcast_id}/episodes/${episode.id}`)
                }} classNameResults="max-h-[min(24rem,calc(100vh-3rem-3rem))]" showBlankState={false} />
            </DialogContent>
        </Dialog>
    )
}
