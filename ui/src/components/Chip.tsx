import { FC, ReactElement } from 'react'

type ChipProps = {
    index: number,
    children?: ReactElement | ReactElement[] | string
}

const defaultChipStyle = {
    gradient: 'bg-linear-to-tl from-green-500 to-teal-500',
    text: 'ui-chip-text-emerald'
}

const chipStyles = [
    {
        gradient: 'bg-linear-to-tl from-blue-500 to-violet-500',
        text: 'ui-chip-text-indigo'
    },
    {
        gradient: 'bg-linear-to-tl from-slate-600 to-slate-300',
        text: 'ui-chip-text-slate'
    },
    {
        gradient: 'bg-linear-to-tl from-blue-700 to-cyan-500',
        text: 'ui-chip-text-sky'
    },
    {
        gradient: 'bg-linear-to-tl from-red-600 to-orange-600',
        text: 'ui-chip-text-orange'
    },
    {
        gradient: 'bg-linear-to-tl from-green-500 to-teal-500',
        text: 'ui-chip-text-emerald'
    },
    {
        gradient: 'bg-linear-to-tl from-gray-400 to-gray-100',
        text: 'ui-chip-text-gray'
    },
    {
        gradient: 'bg-linear-to-tl from-zinc-800 to-zinc-700',
        text: 'ui-chip-text-zinc'
    }
]

export const Chip: FC<ChipProps> = ({ index, children }) => {
    const chipStyle = chipStyles[index % chipStyles.length] ?? defaultChipStyle

    return <span className={`p-px inline-block rounded-sm ${chipStyle.gradient}`}>
        <span className={`ui-chip-label ${chipStyle.text}`}>
            {children}
        </span>
    </span>
}
