import { useQueryClient } from '@tanstack/react-query'
import { useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { CustomInput } from '../components/CustomInput'
import { Heading1 } from '../components/Heading1'
import { LoadingSkeletonSpan } from '../components/ui/LoadingSkeletonSpan'
import type { PodcastTags } from '../models/PodcastTags'
import useCommon from '../store/CommonSlice'
import { $api, client } from '../utils/http'

export const TagsPage = () => {
	const { t } = useTranslation()
	const tags = $api.useQuery('get', '/api/v1/tags')
	const queryClient = useQueryClient()

	return (
		<>
			<Heading1>{t('tag_other')}</Heading1>
			<table className="text-left text-sm text-(--fg-color)">
				<thead>
					<tr className="border-b border-stone-300">
						<th scope="col" className="px-2 py-3 text-(--fg-color)">
							{t('tag_one')}
						</th>
						<th scope="col" className="px-2 py-3 text-(--fg-color)">
							{t('actions')}
						</th>
					</tr>
				</thead>
				<tbody>
					{tags.isLoading || !tags.data
						? Array.from({ length: 5 }).map((value, index, array) => (
								<tr key={index}>
									<td className="px-2 py-4">
										<LoadingSkeletonSpan
											height="30px"
											loading={tags.isLoading}
										/>
									</td>
									<td className="px-2 py-4">
										<LoadingSkeletonSpan
											height="30px"
											loading={tags.isLoading}
										/>
									</td>
								</tr>
							))
						: tags.data.map((tag) => {
								return (
									<tr className="border-b border-stone-300 " key={tag.id}>
										<td className="px-2 py-4 flex items-center text-(--fg-color)">
											<CustomInput
												onBlur={() => {
													client.PUT('/api/v1/tags/{tag_id}', {
														params: {
															path: {
																tag_id: tag.id,
															},
														},
														body: {
															name: tag.name,
															color: tag.color as 'Green' | 'Red' | 'Blue',
														},
													})
												}}
												value={tag.name}
												onChange={(event) => {
													queryClient.setQueryData(
														['get', '/api/v1/tags'],
														(oldData?: PodcastTags[]) => {
															return oldData?.map((t) => {
																if (t.id === tag.id) {
																	return {
																		...t,
																		name: event.target.value,
																	}
																}
															})
														},
													)
												}}
											/>
										</td>
										<td>
											<button
												className="px-2 py-1 text-(--fg-color) rounded-md bg-red-700"
												onClick={() => {
													client
														.DELETE('/api/v1/tags/{tag_id}', {
															params: {
																path: {
																	tag_id: tag.id,
																},
															},
														})
														.then(() => {
															queryClient.setQueryData(
																['get', '/api/v1/tags'],
																(oldData?: PodcastTags[]) =>
																	oldData?.filter(
																		(tagfiltered) => tagfiltered.id !== tag.id,
																	),
															)
														})
												}}
											>
												{t('delete')}
											</button>
										</td>
									</tr>
								)
							})}
				</tbody>
			</table>
		</>
	)
}
