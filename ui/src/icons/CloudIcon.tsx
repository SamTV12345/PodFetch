import type { FC } from 'react'
import { useTranslation } from 'react-i18next'

type CloudIconProps = {
	className?: string
	onClick?: () => void
}

export const CloudIcon: FC<CloudIconProps> = ({ className, onClick }) => {
	const { t } = useTranslation()

	return (
		<div title={t('stream-podcast-episode')!}>
			<svg
				xmlns="http://www.w3.org/2000/svg"
				fill="none"
				viewBox="0 0 24 24"
				strokeWidth={1.5}
				stroke="currentColor"
				className={`w-6 h-6 text-white ${className}`}
				onClick={onClick}
			>
				<path
					strokeLinecap="round"
					strokeLinejoin="round"
					d="M2.25 15a4.5 4.5 0 004.5 4.5H18a3.75 3.75 0 001.332-7.257 3 3 0 00-3.758-3.848 5.25 5.25 0 00-10.233 2.33A4.502 4.502 0 002.25 15z"
				/>
			</svg>
		</div>
	)
}
