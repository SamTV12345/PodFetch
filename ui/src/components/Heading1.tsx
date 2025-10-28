import type { FC } from 'react'

type Heading1Props = {
	children: string
	className?: string
}

export const Heading1: FC<Heading1Props> = ({ children, className = '' }) => {
	return (
		<h1
			className={`font-bold leading-none! text-3xl xs:text-4xl text-(--fg-color) ${className}`}
		>
			{children}
		</h1>
	)
}
