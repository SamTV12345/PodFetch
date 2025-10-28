import type { FC } from 'react'

type RefreshIconProps = {
	onClick: () => void
}

export const RefreshIcon: FC<RefreshIconProps> = ({ onClick }) => {
	return (
		<svg
			xmlns="http://www.w3.org/2000/svg"
			viewBox="0 -960 960 960"
			onClick={onClick}
			className="h-8"
		>
			<path d="M480-160q-133 0-226.5-93.5T160-480q0-133 93.5-226.5T480-800q85 0 149 34.5T740-671v-129h60v254H546v-60h168q-38-60-97-97t-137-37q-109 0-184.5 75.5T220-480q0 109 75.5 184.5T480-220q83 0 152-47.5T728-393h62q-29 105-115 169t-195 64Z" />
		</svg>
	)
}
