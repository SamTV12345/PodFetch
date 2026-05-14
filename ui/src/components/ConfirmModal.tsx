import { FC } from 'react'
import { Dialog, DialogContent, DialogTitle } from '@/components/ui/dialog'
import { CustomButtonPrimary } from './CustomButtonPrimary'
import { CustomButtonSecondary } from './CustomButtonSecondary'

type ConfirmModalProps = {
    open: boolean
    onOpenChange: (open: boolean) => void
    headerText: string
    bodyText: string
    acceptText: string
    rejectText: string
    onAccept: () => void
}

export const ConfirmModal: FC<ConfirmModalProps> = ({ open, onOpenChange, headerText, bodyText, acceptText, rejectText, onAccept }) => {
    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent className="max-w-lg w-full">
                <DialogTitle className="font-bold leading-tight! text-xl xs:text-2xl ui-text mb-4">{headerText}</DialogTitle>

                <div className="mb-4 ui-text">
                    {bodyText}
                </div>
                <div className="flex justify-end gap-3">
                    <CustomButtonSecondary className="border-transparent shadow-none hover:shadow-none text-base ui-text hover:ui-text-hover" onClick={() => onOpenChange(false)}>{rejectText}</CustomButtonSecondary>
                    <CustomButtonPrimary className="bg-(--danger-fg-color) hover:bg-(--danger-fg-color-hover) hover:shadow-(--danger-fg-color-hover) ui-text" onClick={onAccept}>{acceptText}</CustomButtonPrimary>
                </div>
            </DialogContent>
        </Dialog>
    )
}
