import type { FC } from 'react'

type Heading3Props = {
	children: string
	className?: string
}

export const Heading3: FC<Heading3Props> = ({ children, className = '' }) => {
	return (
		<h3 className={`font-bold leading-tight! text-(--fg-color) ${className}`}>
			{children}
		</h3>
	)
}
