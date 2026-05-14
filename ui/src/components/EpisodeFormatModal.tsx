import { FC, ReactNode } from 'react'
import { Dialog, DialogContent, DialogTitle } from '@/components/ui/dialog'
import { Heading2 } from './Heading2'

type EpisodeFormatModalProps = {
    open: boolean
    onOpenChange: (open: boolean) => void
    children: ReactNode | ReactNode[]
    heading: string
}

export const EpisodeFormatModal: FC<EpisodeFormatModalProps> = ({ open, onOpenChange, children, heading }) => {
    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent className="max-w-2xl">
                <div className="mb-4">
                    <DialogTitle render={<Heading2 className="inline align-middle mr-2">{heading}</Heading2>} />
                </div>
                <div className="w-96">
                    {children}
                </div>
            </DialogContent>
        </Dialog>
    )
}
