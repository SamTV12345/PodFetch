import { useTranslation } from 'react-i18next'
import usePlaylist from '../store/PlaylistSlice'
import { client } from '../utils/http'
import { CustomButtonPrimary } from './CustomButtonPrimary'

export const PlaylistSubmitViewer = () => {
	const { t } = useTranslation()
	const currentPlaylistToEdit = usePlaylist(
		(state) => state.currentPlaylistToEdit,
	)
	const playlists = usePlaylist((state) => state.playlist)
	const setCreatePlaylistOpen = usePlaylist(
		(state) => state.setCreatePlaylistOpen,
	)
	const setPlaylist = usePlaylist((state) => state.setPlaylist)

	const savePlaylist = () => {
		const idsToMap = currentPlaylistToEdit!.items.map((item) => {
			return {
				episode: item.podcastEpisode.id,
			}
		})

		client
			.POST('/api/v1/playlist', {
				body: {
					name: currentPlaylistToEdit?.name!,
					items: idsToMap,
				},
			})
			.then((v) => {
				setPlaylist([...playlists, v.data!])
				setCreatePlaylistOpen(false)
			})
	}

	return (
		<>
			<CustomButtonPrimary
				type="submit"
				className="float-right"
				onClick={() => {
					savePlaylist()
				}}
			>
				{currentPlaylistToEdit?.id === '-1'
					? t('create-playlist')
					: t('update-playlist')}
			</CustomButtonPrimary>
			<br />
		</>
	)
}
