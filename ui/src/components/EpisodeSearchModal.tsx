import { useState } from 'react'
import { createPortal } from 'react-dom'
import { useNavigate } from 'react-router-dom'
import { useCtrlPressed, useKeyDown } from '../hooks/useKeyDown'
import { EpisodeSearch } from './EpisodeSearch'

export const EpisodeSearchModal = () => {
	const [open, setOpen] = useState<boolean>(false)
	const navigate = useNavigate()

	useCtrlPressed(() => {
		setOpen(!open)
		if (open) {
			document.getElementById('search-input')?.blur()
		} else {
			document.getElementById('search-input')?.focus()
		}
	}, ['f'])

	useKeyDown(() => {
		setOpen(false)
		document.getElementById('search-input')?.blur()
	}, ['Escape'])

	return createPortal(
		<div
			id="defaultModal"
			tabIndex={-1}
			aria-hidden="true"
			onClick={() => setOpen(false)}
			className={`grid place-items-center fixed inset-0 bg-[rgba(0,0,0,0.5)] backdrop-blur-sm overflow-x-hidden overflow-y-auto z-30 ${open ? 'opacity-100' : 'opacity-0 pointer-events-none'}`}
		>
			<div
				className={`bg-(--bg-color) max-h-screen max-w-4xl p-4 rounded-2xl shadow-[0_4px_16px_rgba(0,0,0,var(--shadow-opacity))] w-full`}
				onClick={(e) => e.stopPropagation()}
			>
				{/*
                Max-height of search results section should be the lesser of:
                    - 24rem
                    - Or, for when screen height is smaller: viewport height - vertical padding/spacing - height of search field
                */}
				<EpisodeSearch
					onClickResult={(episode) => {
						setOpen(false)
						navigate(`/podcasts/${episode.podcast_id}/episodes/${episode.id}`)
					}}
					classNameResults="max-h-[min(24rem,calc(100vh-3rem-3rem))]"
					showBlankState={false}
				/>
			</div>
		</div>,
		document.getElementById('modal') as Element,
	)
}
