import { FC } from 'react'
import * as Dialog from '@radix-ui/react-dialog'
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
        <Dialog.Root open={open} onOpenChange={onOpenChange}>
            <Dialog.Portal>
                <Dialog.Overlay className="fixed inset-0 bg-[rgba(0,0,0,0.5)] backdrop-blur-sm z-30" />
                <Dialog.Content className="fixed inset-0 z-40 flex items-center justify-center p-4">
                    <div className="relative ui-surface max-w-lg p-8 rounded-2xl shadow-[0_4px_16px_rgba(0,0,0,var(--shadow-opacity))] w-full">
                        <Dialog.Title className="font-bold leading-tight! text-xl xs:text-2xl ui-text mb-4">{headerText}</Dialog.Title>
                        <Dialog.Close className="absolute top-4 right-4 bg-transparent">
                            <span className="material-symbols-outlined ui-modal-close hover:ui-modal-close-hover">close</span>
                        </Dialog.Close>

                        <div className="mb-4 ui-text">
                            {bodyText}
                        </div>
                        <div className="flex justify-end gap-3">
                            <CustomButtonSecondary className="border-transparent shadow-none hover:shadow-none text-base ui-text hover:ui-text-hover" onClick={() => onOpenChange(false)}>{rejectText}</CustomButtonSecondary>
                            <CustomButtonPrimary className="bg-(--danger-fg-color) hover:bg-(--danger-fg-color-hover) hover:shadow-(--danger-fg-color-hover) ui-text" onClick={onAccept}>{acceptText}</CustomButtonPrimary>
                        </div>
                    </div>
                </Dialog.Content>
            </Dialog.Portal>
        </Dialog.Root>
    )
}
