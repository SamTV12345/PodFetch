import { FC, ReactNode } from 'react'
import * as Dialog from '@radix-ui/react-dialog'
import { useTranslation } from 'react-i18next'
import { Heading2 } from './Heading2'
import 'material-symbols/outlined.css'

type EpisodeFormatModalProps = {
    open: boolean
    onOpenChange: (open: boolean) => void
    children: ReactNode | ReactNode[]
    heading: string
}

export const EpisodeFormatModal: FC<EpisodeFormatModalProps> = ({ open, onOpenChange, children, heading }) => {
    const { t } = useTranslation()

    return (
        <Dialog.Root open={open} onOpenChange={onOpenChange}>
            <Dialog.Portal>
                <Dialog.Overlay className="fixed inset-0 bg-[rgba(0,0,0,0.5)] backdrop-blur-sm z-30" />
                <Dialog.Content className="fixed inset-0 z-40 flex items-center justify-center p-4">
                    <div className="relative ui-surface max-w-2xl p-8 rounded-2xl shadow-[0_4px_16px_rgba(0,0,0,var(--shadow-opacity))]">
                        <Dialog.Close className="absolute top-4 right-4 bg-transparent">
                            <span className="material-symbols-outlined ui-modal-close hover:ui-modal-close-hover">close</span>
                            <span className="sr-only">{t('closeModal')}</span>
                        </Dialog.Close>
                        <div className="mb-4">
                            <Dialog.Title asChild>
                                <Heading2 className="inline align-middle mr-2">{heading}</Heading2>
                            </Dialog.Title>
                        </div>
                        <div className="w-96">
                            {children}
                        </div>
                    </div>
                </Dialog.Content>
            </Dialog.Portal>
        </Dialog.Root>
    )
}
