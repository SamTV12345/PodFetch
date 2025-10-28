import { type FC, useState } from 'react'
import { createPortal } from 'react-dom'
import type { AudioAmplifier } from '../models/AudioAmplifier'
import useCommon from '../store/CommonSlice'
import { removeHTML } from '../utils/Utilities'
import { PlayerProgressBar } from './PlayerProgressBar'
import { PlayerTimeControls } from './PlayerTimeControls'
import { PlayerVolumeSlider } from './PlayerVolumeSlider'
import 'material-symbols/outlined.css'
import { useTranslation } from 'react-i18next'
import useAudioPlayer from '../store/AudioPlayerSlice'
import { PodcastEpisodeChapterTable } from './PodcastEpisodeChapterTable'

type DetailedAudioPlayerProps = {
	audioAmplifier: AudioAmplifier | undefined
	setAudioAmplifier: (audioAmplifier: AudioAmplifier | undefined) => void
}

export const DetailedAudioPlayer: FC<DetailedAudioPlayerProps> = ({
	audioAmplifier,
}) => {
	const setDetailedAudioPlayerOpen = useCommon(
		(state) => state.setDetailedAudioPlayerOpen,
	)
	const currentPodcast = useAudioPlayer((state) => state.currentPodcast)
	const currentPodcastEpisode = useAudioPlayer(
		(state) => state.loadedPodcastEpisode,
	)
	const { t } = useTranslation()
	const [selectedTab, setSelectedTab] = useState<'description' | 'chapters'>(
		'description',
	)

	return createPortal(
		<div
			tabIndex={-1}
			aria-hidden="true"
			className="grid grid-rows-[1fr_auto] fixed inset-0 bg-(--bg-color) md:h-full overflow-x-hidden overflow-y-auto z-30"
			onClick={(event) => event.stopPropagation()}
		>
			<span
				className="material-symbols-outlined absolute top-2 left-2 cursor-pointer text-4xl text-(--fg-color) hover:text-(--fg-color-hover)"
				onClick={() => setDetailedAudioPlayerOpen(false)}
			>
				close_fullscreen
			</span>

			{/* Episode information */}
			<div
				className="
        grid
        grid-cols-[1fr] grid-rows-[auto_1fr]
        md:grid-cols-[auto_1fr] md:grid-rows-1
        md:items-start gap-4 xs:gap-8 md:gap-10
        px-4 py-8 xs:px-8 md:px-12
        overflow-hidden
        "
			>
				{/* Thumbnail and titles */}
				<div className="flex flex-col xs:flex-row items-center gap-4 md:block h-full place-content-center">
					<div
						className="aspect-square bg-center bg-cover md:mb-4 rounded-xl h-40 md:h-60 lg:h-80"
						style={{
							backgroundImage: `url("${currentPodcastEpisode?.podcastEpisode.local_image_url}")`,
						}}
					></div>

					<div className="text-center xs:text-left">
						<span className="block font-bold leading-tight mb-2 text-xl lg:text-2xl text-(--fg-color)">
							{currentPodcastEpisode?.podcastEpisode.name}
						</span>
						<span className="block lg:text-lg text-(--fg-color)">
							{currentPodcast?.name}
						</span>
					</div>
				</div>

				{/* Description with scroll */}
				<div className="">
					<ul className="flex flex-wrap gap-2 border-b border-(--border-color) mb-6 text-(--fg-secondary-color)">
						<li
							onClick={() => setSelectedTab('description')}
							className={`cursor-pointer inline-block px-2 py-4 ${selectedTab === 'description' && 'border-b-2 border-(--accent-color) text-(--accent-color)'}`}
						>
							{t('description')}
						</li>
						<li
							onClick={() => setSelectedTab('chapters')}
							className={`cursor-pointer inline-block px-2 py-4 ${selectedTab === 'chapters' && 'border-b-2 border-(--accent-color) text-(--accent-color)'}`}
						>
							{t('chapters')}
						</li>
					</ul>

					<div className="overflow-y-auto max-h-11/12">
						{currentPodcastEpisode &&
							(selectedTab === 'description' ? (
								<div
									className="xs:text-lg md:text-xl lg:text-2xl text-(--fg-color)"
									dangerouslySetInnerHTML={
										currentPodcastEpisode?.podcastEpisode.description
											? removeHTML(
													currentPodcastEpisode.podcastEpisode.description,
												)
											: { __html: '' }
									}
								/>
							) : (
								<PodcastEpisodeChapterTable
									podcastEpisode={currentPodcastEpisode.podcastEpisode}
								/>
							))}
					</div>
				</div>
			</div>

			{/* Player */}
			<div className="bg-(--bg-color) px-2 xs:p-4">
				<PlayerProgressBar
					className="mb-2"
					currentPodcastEpisode={currentPodcastEpisode}
				/>

				<div
					className="
                grid
                grid-col-1 xs:grid-cols-[0_1fr_12rem] md:grid-cols-[12rem_1fr_12rem]
                justify-items-center
                px-3 xs:px-4 md:px-10
            "
				>
					<div></div>
					<PlayerTimeControls currentPodcastEpisode={currentPodcastEpisode} />
					<PlayerVolumeSlider audioAmplifier={audioAmplifier} />
				</div>
			</div>
		</div>,
		document.getElementById('modal') as Element,
	)
}
