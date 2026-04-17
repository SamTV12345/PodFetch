import { FC, useState } from 'react'
import * as Dialog from '@radix-ui/react-dialog'
import { useTranslation } from 'react-i18next'
import { AddHeader } from './AddHeader'
import { AddTypes } from '../models/AddTypes'
import { FeedURLComponent } from './FeedURLComponent'
import { OpmlAdd } from './OpmlAdd'
import { ProviderImportComponent } from './ProviderImportComponent'
import { $api } from '../utils/http'

type AddPodcastModalProps = {
    open: boolean
    onOpenChange: (open: boolean) => void
}

export const AddPodcastModal: FC<AddPodcastModalProps> = ({ open, onOpenChange }) => {
    const { t } = useTranslation()
    const [selectedSearchType, setSelectedSearchType] = useState<AddTypes>(AddTypes.ITUNES)
    const configModel = $api.useQuery('get', '/api/v1/sys/config')

    return (
        <Dialog.Root open={open} onOpenChange={onOpenChange}>
            <Dialog.Portal>
                <Dialog.Overlay className="fixed inset-0 bg-[rgba(0,0,0,0.5)] backdrop-blur-sm z-30" />
                <Dialog.Content className="fixed inset-0 z-40 flex items-center justify-center p-4">
                    <div className="relative ui-surface max-w-lg p-8 rounded-2xl shadow-[0_4px_16px_rgba(0,0,0,var(--shadow-opacity))] w-full">
                        <Dialog.Title className="font-bold leading-tight! text-xl xs:text-2xl ui-text mb-4">{t('add-podcast')}</Dialog.Title>
                        <Dialog.Close className="absolute top-4 right-4 bg-transparent">
                            <span className="material-symbols-outlined ui-modal-close hover:ui-modal-close-hover">close</span>
                        </Dialog.Close>

                        {configModel.data && <AddHeader selectedSearchType={selectedSearchType} setSelectedSearchType={setSelectedSearchType} configModel={configModel.data} />}

                        {selectedSearchType !== AddTypes.OPML && selectedSearchType !== AddTypes.FEED &&
                            <ProviderImportComponent selectedSearchType={selectedSearchType} onClose={() => onOpenChange(false)} />
                        }
                        {selectedSearchType === AddTypes.OPML &&
                            <OpmlAdd selectedSearchType={selectedSearchType} />
                        }
                        {selectedSearchType === AddTypes.FEED &&
                            <FeedURLComponent />
                        }
                    </div>
                </Dialog.Content>
            </Dialog.Portal>
        </Dialog.Root>
    )
}
