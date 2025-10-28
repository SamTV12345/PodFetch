import { type FC, useState } from 'react'
import { useTranslation } from 'react-i18next'
import type { components } from '../../schema'
import { EmptyResultIcon } from '../icons/EmptyResultIcon'
import { $api, client } from '../utils/http'
import { formatTime, removeHTML } from '../utils/Utilities'
import { useDebounce } from '../utils/useDebounce'
import { CustomInput } from './CustomInput'
import { Spinner } from './Spinner'

type EpisodeSearchProps = {
	classNameResults?: string
	onClickResult?: (e: components['schemas']['PodcastEpisodeDto']) => void
	showBlankState?: boolean
}

export const EpisodeSearch: FC<EpisodeSearchProps> = ({
	classNameResults = '',
	onClickResult = () => null,
	showBlankState = true,
}) => {
	const [searching, setSearching] = useState<boolean>()
	const [searchName, setSearchName] = useState<string>('')
	const [searchResults, setSearchResults] = useState<
		components['schemas']['PodcastEpisodeDto'][]
	>([])
	const { t } = useTranslation()
	const { data, isLoading } = $api.useQuery('get', '/api/v1/users/{username}', {
		params: {
			path: {
				username: 'me',
			},
		},
	})

	const performSearch = () => {
		if (searchName.trim().length > 0) {
			setSearching(true)

			client
				.GET('/api/v1/podcasts/{podcast}/query', {
					params: {
						path: {
							podcast: searchName,
						},
					},
				})
				.then((v) => {
					if (!v.data) return
					setSearchResults(v.data)
					setSearching(false)
				})
		}
	}

	useDebounce(performSearch, 500, [searchName])

	if (isLoading || !data) {
		return (
			<div className="grid place-items-center p-10">
				<Spinner className="w-12 h-12" />
			</div>
		)
	}

	return (
		<>
			{/* Search field */}
			<div className="flex items-center relative">
				<CustomInput
					className="pl-10 w-full"
					id="search-input"
					onChange={(v) => setSearchName(v.target.value)}
					placeholder={t('search-episodes')}
					type="text"
					value={searchName}
				/>

				<span className="material-symbols-outlined absolute left-2 text-(--input-icon-color)">
					search
				</span>
			</div>

			{
				/* Results */
				searching ? (
					<div className="grid place-items-center p-10">
						<Spinner className="w-12 h-12" />
					</div>
				) : searchResults.length === 0 ? (
					<div className="grid place-items-center">
						{searchName ? (
							<span className="p-8 text-(--fg-secondary-color)">
								{t('no-results-found-for')} "
								<span className="text-(--fg-color)">{searchName}</span>"
							</span>
						) : (
							showBlankState && <EmptyResultIcon />
						)}
					</div>
				) : (
					<ul
						className={`flex flex-col gap-10 overflow-y-auto my-4 px-8 py-6 scrollbox-y ${classNameResults}`}
					>
						{searchResults.map((episode) => (
							<li
								className="flex gap-4 cursor-pointer group"
								key={episode.id}
								onClick={() => {
									onClickResult(episode)
								}}
							>
								{/* Thumbnail */}
								<img
									alt={episode.name}
									className="
                                hidden xs:block
                                rounded-lg w-32 transition-shadow group-hover:shadow-[0_4px_32px_rgba(0,0,0,0.3)]
                            "
									src={episode.image_url}
								/>

								{/* Information */}
								<div className="flex flex-col gap-2">
									<span className="text-sm text-(--fg-secondary-color)">
										{formatTime(episode.date_of_recording)}
									</span>
									<span className="font-bold leading-tight text-(--fg-color) transition-color group-hover:text-(--fg-color-hover)">
										{episode.name}
									</span>
									<div
										className="leading-[1.75] line-clamp-3 text-sm text-(--fg-color)"
										dangerouslySetInnerHTML={removeHTML(episode.description)}
									></div>
								</div>
							</li>
						))}
					</ul>
				)
			}
		</>
	)
}
