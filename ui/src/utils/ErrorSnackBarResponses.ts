import type { TFunction } from 'i18next'
import { enqueueSnackbar } from 'notistack'

export const handleAddPodcast = (
	resp: number | null,
	podcast: string,
	t: TFunction,
) => {
	if (resp === null) {
		enqueueSnackbar(t('error'), { variant: 'error' })
		return
	}
	switch (resp) {
		case 409:
			enqueueSnackbar(
				t('already-added', {
					name: podcast,
				}),
				{ variant: 'error' },
			)
			break
		case 403:
			enqueueSnackbar(t('not-admin-or-uploader'), { variant: 'error' })
			break
		default:
			enqueueSnackbar(t('not-admin-or-uploader'), { variant: 'error' })
	}
}
