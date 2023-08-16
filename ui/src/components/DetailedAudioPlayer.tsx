import { FC, RefObject } from 'react'
import { createPortal } from 'react-dom'
import { useAppDispatch, useAppSelector } from '../store/hooks'
import { setDetailedAudioPlayerOpen } from '../store/CommonSlice'
import { removeHTML } from '../utils/Utilities'
import { AudioAmplifier } from '../models/AudioAmplifier'
import { PlayerTimeControls } from './PlayerTimeControls'
import { PlayerProgressBar } from './PlayerProgressBar'
import { PlayerVolumeSlider } from './PlayerVolumeSlider'
import 'material-symbols/outlined.css'

type DetailedAudioPlayerProps = {
    refItem: RefObject<HTMLAudioElement>,
    audioAmplifier: AudioAmplifier | undefined,
    setAudioAmplifier: (audioAmplifier: AudioAmplifier | undefined) => void
}

export const DetailedAudioPlayer: FC<DetailedAudioPlayerProps> = ({ refItem, audioAmplifier }) => {
    const dispatch = useAppDispatch()
    const selectedPodcast = useAppSelector(state => state.audioPlayer.currentPodcastEpisode)
    const currentPodcast = useAppSelector(state => state.audioPlayer.currentPodcast)

    return createPortal(
        <div tabIndex={-1} aria-hidden="true" className="grid grid-rows-[1fr_auto] fixed inset-0 bg-[--bg-color] md:h-full overflow-x-hidden overflow-y-auto z-30" onClick={event => event.stopPropagation()}>
            <span className="material-symbols-outlined absolute top-2 left-2 cursor-pointer text-4xl text-[--fg-color] hover:text-[--fg-color-hover]" onClick={() => dispatch(setDetailedAudioPlayerOpen(false))}>close_fullscreen</span>

            {/* Episode information */}
            <div className="
            grid
            grid-cols-[1fr] grid-rows-[auto_1fr]
            md:grid-cols-[auto_auto] md:grid-rows-1
            md:items-center gap-4 xs:gap-8 md:gap-10
            px-4 py-8 xs:px-8 md:px-12
            ">
                {/* Thumbnail and titles need special positioning to vertically align thumbnail with description */}
                <div className="flex flex-col xs:flex-row items-center gap-4 md:block md:relative md:mb-10">
                    <div className="aspect-square bg-center bg-cover md:mb-4 rounded-xl h-40 md:h-60 lg:h-80" style={{ backgroundImage: `url("${selectedPodcast?.local_image_url}")` }}></div>

                    <div className="md:absolute text-center xs:text-left">
                        <span className="block font-bold leading-tight mb-2 text-xl lg:text-2xl text-[--fg-color]">{selectedPodcast?.name}</span>
                        <span className="block lg:text-lg text-[--fg-color]">{currentPodcast && currentPodcast.name}</span>
                    </div>
                </div>

                {/* Description */}
                <div className="md:max-h-60 lg:max-h-80 md:mb-10 overflow-y-auto xs:text-lg md:text-xl lg:text-2xl text-[--fg-color]" dangerouslySetInnerHTML={selectedPodcast?.description ? removeHTML(selectedPodcast.description) : { __html: '' }} />
            </div>

            {/* Player */}
            <div className="bg-[--bg-color] px-2 xs:p-4">
                <PlayerProgressBar audioplayerRef={refItem} className="mb-2" />

                <div className="
                    grid
                    grid-col-1 xs:grid-cols-[0_1fr_12rem] md:grid-cols-[12rem_1fr_12rem]
                    justify-items-center
                    px-3 xs:px-4 md:px-10
                ">
                    <div></div>
                    <PlayerTimeControls refItem={refItem} />
                    <PlayerVolumeSlider refItem={refItem} audioAmplifier={audioAmplifier} />
                </div>
            </div>

        </div>,document.getElementById('modal')!
    )
}
