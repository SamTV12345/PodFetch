import type { FC } from 'react'
import { useTranslation } from 'react-i18next'
import { cn } from '../lib/utils'
import { Spinner } from './Spinner'

type LoadingSpinnerProps = {
	className?: string
}

export const Loading: FC<LoadingSpinnerProps> = ({ className }) => {
	const { t } = useTranslation()

	return (
		<div className={cn('grid place-items-center h-full w-full', className)}>
			<div className="flex items-center gap-3" role="status">
				<Spinner />

				<span className="text-(--fg-color)">{t('loading')}...</span>
			</div>
		</div>
	)
}
