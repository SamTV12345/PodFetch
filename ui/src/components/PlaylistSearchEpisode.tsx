import { type DragEvent, useState } from 'react'
import { useTranslation } from 'react-i18next'
import type { components } from '../../schema'
import { PodcastEpisode } from '../store/CommonSlice'
import usePlaylist from '../store/PlaylistSlice'
import { EpisodeSearch } from './EpisodeSearch'

export const PlaylistSearchEpisode = () => {
	const [itemCurrentlyDragged, setItemCurrentlyDragged] =
		useState<components['schemas']['PodcastEpisodeDto']>()
	const { t } = useTranslation()
	const currentPlayListToEdit = usePlaylist(
		(state) => state.currentPlaylistToEdit,
	)
	const setCurrentPlaylistToEdit = usePlaylist(
		(state) => state.setCurrentPlaylistToEdit,
	)
	const handleDragStart = (
		dragItem: components['schemas']['PodcastEpisodeDto'],
		index: number,
		event: DragEvent<HTMLTableRowElement>,
	) => {
		event.dataTransfer.setData('text/plain', index.toString())
		setItemCurrentlyDragged(dragItem)
	}

	return (
		<>
			<EpisodeSearch
				onClickResult={(e) => {
					setCurrentPlaylistToEdit({
						id: currentPlayListToEdit!.id,
						name: currentPlayListToEdit!.name,
						items: [...currentPlayListToEdit!.items, { podcastEpisode: e }],
					})
				}}
				classNameResults="max-h-[min(20rem,calc(100vh-3rem-3rem))]"
				showBlankState={false}
			/>
			<div className={`scrollbox-x  p-2`}>
				<table className="text-left text-sm text-stone-900 w-full overflow-y-auto text-(--fg-color)">
					<thead>
						<tr className="border-b border-stone-300">
							<th scope="col" className="pr-2 py-3 text-(--fg-color)">
								#
							</th>
							<th scope="col" className="px-2 py-3 text-(--fg-color)">
								{t('episode-name')}
							</th>
							<th scope="col" className="px-2 py-3 text-(--fg-color)">
								{t('actions')}
							</th>
						</tr>
					</thead>
					<tbody className="">
						{currentPlayListToEdit?.items.map((item, index) => {
							return (
								<tr
									className="border-2 border-white"
									draggable
									onDrop={(e) => {
										e.preventDefault()
										const dropIndex = index
										const dragIndex = parseInt(
											e.dataTransfer.getData('text/plain'),
										)

										const newItems = [...currentPlayListToEdit!.items]
										const dragItem = newItems[dragIndex]!
										newItems.splice(dragIndex, 1)
										newItems.splice(dropIndex, 0, dragItem)
										setCurrentPlaylistToEdit({
											name: currentPlayListToEdit!.name,
											id: currentPlayListToEdit!.id,
											items: newItems,
										})
									}}
									onDragOver={(e) =>
										item.podcastEpisode.id != itemCurrentlyDragged?.id &&
										e.preventDefault()
									}
									onDragStart={(e) =>
										handleDragStart(item.podcastEpisode, index, e)
									}
								>
									<td className="text-(--fg-color) p-2">{index}</td>
									<td className="text-(--fg-color)">
										{item.podcastEpisode.name}
									</td>
									<td>
										<button
											className="flex text-red-700 hover:text-red-500"
											onClick={(e) => {
												e.preventDefault()
												setCurrentPlaylistToEdit({
													name: currentPlayListToEdit!.name,
													id: currentPlayListToEdit!.id,
													items: currentPlayListToEdit!.items.filter(
														(i) =>
															i.podcastEpisode.id !== item.podcastEpisode.id,
													),
												})
											}}
										>
											<span className="material-symbols-outlined mr-1">
												delete
											</span>
											{t('delete')}
										</button>
									</td>
								</tr>
							)
						})}
					</tbody>
				</table>
			</div>
		</>
	)
}
