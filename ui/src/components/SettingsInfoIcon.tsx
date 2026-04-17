import { FC } from 'react'
import * as Dialog from '@radix-ui/react-dialog'
import { useTranslation } from 'react-i18next'

type SettingsInfoIconProps = {
    headerKey: string
    textKey: string
    className?: string
}

export const SettingsInfoIcon: FC<SettingsInfoIconProps> = ({ textKey, headerKey, className }) => {
    const { t } = useTranslation()

    return (
        <Dialog.Root>
            <Dialog.Trigger asChild>
                <button type="button">
                    <span className="material-symbols-outlined pointer active:scale-95">info</span>
                </button>
            </Dialog.Trigger>
            <Dialog.Portal>
                <Dialog.Overlay className="fixed inset-0 bg-[rgba(0,0,0,0.5)] backdrop-blur-sm z-30" />
                <Dialog.Content className="fixed inset-0 z-40 flex items-center justify-center p-4">
                    <div className="relative ui-surface max-w-2xl p-8 rounded-2xl shadow-[0_4px_16px_rgba(0,0,0,var(--shadow-opacity))]">
                        <Dialog.Title className="font-bold leading-tight! text-xl xs:text-2xl ui-text mb-4">
                            {t(headerKey)}
                        </Dialog.Title>
                        <Dialog.Close className="absolute top-4 right-4 bg-transparent">
                            <span className="material-symbols-outlined ui-modal-close hover:ui-modal-close-hover">close</span>
                        </Dialog.Close>
                        <p className="leading-[1.75] text-sm ui-text">{t(textKey)}</p>
                    </div>
                </Dialog.Content>
            </Dialog.Portal>
        </Dialog.Root>
    )
}
