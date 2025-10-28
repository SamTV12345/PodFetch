import { useQueryClient } from '@tanstack/react-query'
import { enqueueSnackbar } from 'notistack'
import { type FC, Fragment, useState } from 'react'
import { useTranslation } from 'react-i18next'
import type { components } from '../../schema'
import useCommon from '../store/CommonSlice'
import useModal from '../store/ModalSlice'
import { $api, client } from '../utils/http'
import { CustomButtonSecondary } from './CustomButtonSecondary'
import { CustomCheckbox } from './CustomCheckbox'
import { LoadingSkeletonSpan } from './ui/LoadingSkeletonSpan'

export const SettingsPodcastDelete: FC = () => {
	const { t } = useTranslation()
	const podcasts = $api.useQuery('get', '/api/v1/podcasts')
	const setModalOpen = useModal((state) => state.setOpenModal)
	const setConfirmModalData = useCommon((state) => state.setConfirmModalData)
	const queryClient = useQueryClient()
	const [selectedPodcasts, setSelectedPodcasts] = useState<
		components['schemas']['PodcastDto'][]
	>([])

	const deletePodcast = (withFiles: boolean) => {
		selectedPodcasts.forEach((p) => {
			client
				.DELETE('/api/v1/podcasts/{id}', {
					body: {
						delete_files: withFiles,
					},
					params: {
						path: {
							id: p.id,
						},
					},
				})
				.then(() => {
					enqueueSnackbar(t('podcast-deleted', { name: p.name }), {
						variant: 'success',
					})
					for (const queryKey of queryClient
						.getQueryCache()
						.getAll()
						.map((q) => q.queryKey)) {
						if (
							(queryKey[0] === 'get' &&
								typeof queryKey[1] === 'string' &&
								queryKey[1] === '/api/v1/podcasts/search') ||
							(queryKey[0] === 'get' &&
								typeof queryKey[1] === 'string' &&
								queryKey[1] === '/api/v1/podcasts')
						) {
							queryClient.setQueryData(
								queryKey,
								(oldData: components['schemas']['PodcastDto'][]) => {
									return oldData.filter((pod) => pod.id !== p.id)
								},
							)
						}
					}
				})
		})
	}

	return (
		<div className="grid grid-cols-1 lg:grid-cols-[1fr_auto_auto] items-center gap-6">
			{podcasts.isLoading || !podcasts.data ? (
				<LoadingSkeletonSpan
					height="30px"
					text={''}
					loading={podcasts.isLoading}
				/>
			) : (
				<CustomCheckbox
					value={selectedPodcasts.length === podcasts.data.length}
					onChange={(v) => {
						if (v.valueOf() === true) {
							setSelectedPodcasts(podcasts.data)
						} else {
							setSelectedPodcasts([])
						}
					}}
				/>
			)}

			<CustomButtonSecondary
				disabled={selectedPodcasts.length === 0}
				onClick={() => {
					setConfirmModalData({
						headerText: t('delete-podcast-with-files'),
						onAccept: () => {
							deletePodcast(true)
							setModalOpen(false)
						},
						onReject: () => {
							setModalOpen(false)
						},
						acceptText: t('delete-podcast-confirm'),
						rejectText: t('cancel'),
						bodyText: t('delete-podcast-with-files-body', {
							name: [selectedPodcasts.map((a) => a.name).join(', ')],
						}),
					})
					setModalOpen(true)
				}}
			>
				{t('delete-podcast-with-files')}
			</CustomButtonSecondary>

			<CustomButtonSecondary
				disabled={selectedPodcasts.length === 0}
				onClick={() => {
					setConfirmModalData({
						headerText: t('delete-podcast-without-files'),
						onAccept: () => {
							deletePodcast(false)
							setModalOpen(false)
						},
						onReject: () => {
							setModalOpen(false)
						},
						acceptText: t('delete-podcast-confirm'),
						rejectText: t('cancel'),
						bodyText: t('delete-podcast-without-files-body', {
							name: [selectedPodcasts.map((a) => a.name).join(', ')],
						}),
					})
					setModalOpen(true)
				}}
			>
				{t('delete-podcast-without-files')}
			</CustomButtonSecondary>
			<hr className="col-span-1 lg:col-span-3" />
			{podcasts.isLoading || !podcasts.data
				? Array.from({ length: 10 }).map((_value, index, _array) => {
						return (
							// biome-ignore lint: This is just the skeleton
							<Fragment key={index}>
								<LoadingSkeletonSpan
									text={''}
									loading={podcasts.isLoading}
									height="30px"
								/>
								<LoadingSkeletonSpan
									text={''}
									loading={podcasts.isLoading}
									height="30px"
								/>
							</Fragment>
						)
					})
				: podcasts.data.map((p) => (
						<Fragment key={p.id}>
							<CustomCheckbox
								value={selectedPodcasts.includes(p)}
								onChange={(v) => {
									const isChecked = v.valueOf() as boolean

									if (isChecked) {
										if (!selectedPodcasts.includes(p)) {
											setSelectedPodcasts([...selectedPodcasts, p])
										}
									} else {
										setSelectedPodcasts(
											selectedPodcasts.filter((pod) => pod !== p),
										)
									}
								}}
							/>
							<span className="xs:col-span-2 lg:col-span-2 text-(--fg-color)">
								{p.name}
							</span>
						</Fragment>
					))}
		</div>
	)
}
