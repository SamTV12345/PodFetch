import type { FC } from 'react'
import { Skeleton } from './skeleton'

type LoadingSkeletonProps = {
	loading?: boolean
	text: string | undefined | number
}

export const LoadingSkeletonDD: FC<LoadingSkeletonProps> = ({
	loading,
	text,
}) => {
	return (
		<dd className="text-(--fg-secondary-color)">
			{loading === true ? <Skeleton style={{ height: '100%' }} /> : text}
		</dd>
	)
}
