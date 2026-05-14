import { FC, useState } from 'react'
import { Dialog, DialogContent, DialogTitle } from '@/components/ui/dialog'
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
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent className="max-w-lg w-full">
                <DialogTitle className="font-bold leading-tight! text-xl xs:text-2xl ui-text mb-4">{t('add-podcast')}</DialogTitle>

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
            </DialogContent>
        </Dialog>
    )
}
