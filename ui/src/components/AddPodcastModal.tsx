import { type FC, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { AddTypes } from '../models/AddTypes'
import { $api } from '../utils/http'
import { AddHeader } from './AddHeader'
import { FeedURLComponent } from './FeedURLComponent'
import { Modal } from './Modal'
import { OpmlAdd } from './OpmlAdd'
import { ProviderImportComponent } from './ProviderImportComponent'

export const AddPodcastModal: FC = () => {
	const { t } = useTranslation()
	const [selectedSearchType, setSelectedSearchType] = useState<AddTypes>(
		AddTypes.ITUNES,
	)
	const configModel = $api.useQuery('get', '/api/v1/sys/config')

	return (
		<Modal
			onCancel={() => {}}
			onAccept={() => {}}
			headerText={t('add-podcast')}
			onDelete={() => {}}
			cancelText={'Abbrechen'}
			acceptText={'Hinzufügen'}
		>
			{configModel.data && (
				<AddHeader
					selectedSearchType={selectedSearchType}
					setSelectedSearchType={setSelectedSearchType}
					configModel={configModel.data}
				/>
			)}

			{selectedSearchType !== AddTypes.OPML &&
				selectedSearchType !== AddTypes.FEED && (
					<ProviderImportComponent selectedSearchType={selectedSearchType} />
				)}
			{selectedSearchType === AddTypes.OPML && <OpmlAdd />}
			{selectedSearchType === AddTypes.FEED && <FeedURLComponent />}
		</Modal>
	)
}
