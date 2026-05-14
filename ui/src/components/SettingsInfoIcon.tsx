import { FC } from 'react'
import { Dialog, DialogContent, DialogTitle, DialogTrigger } from '@/components/ui/dialog'
import { useTranslation } from 'react-i18next'

type SettingsInfoIconProps = {
    headerKey: string
    textKey: string
    className?: string
}

export const SettingsInfoIcon: FC<SettingsInfoIconProps> = ({ textKey, headerKey, className }) => {
    const { t } = useTranslation()

    return (
        <Dialog>
            <DialogTrigger
                render={
                    <button
                        type="button"
                        className={`inline-flex items-center align-middle ml-1 cursor-pointer ${className ?? ''}`}
                    >
                        <span className="material-symbols-outlined text-base leading-none active:scale-95">info</span>
                    </button>
                }
            />
            <DialogContent className="max-w-2xl">
                <DialogTitle className="font-bold leading-tight! text-xl xs:text-2xl ui-text mb-4">
                    {t(headerKey)}
                </DialogTitle>
                <p className="leading-[1.75] text-sm ui-text">{t(textKey)}</p>
            </DialogContent>
        </Dialog>
    )
}
