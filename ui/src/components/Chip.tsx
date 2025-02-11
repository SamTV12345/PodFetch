import { FC, ReactElement } from 'react'

type ChipProps = {
    index: number,
    children?: ReactElement | ReactElement[] | string
}

export const Chip: FC<ChipProps> = ({ index, children }) => {
    switch (index % 7) {
        case 0:
            return <span
                className="p-px inline-block rounded-sm bg-linear-to-tl from-blue-500 to-violet-500">
                    <span className="block bg-(--bg-color) leading-none p-1.5 rounded-sm text-center text-xs text-indigo-700 dark:text-indigo-400 whitespace-nowrap">
                        {children}
                    </span>
                </span>
        case 1:
            return <span
                className="p-px inline-block rounded-sm bg-linear-to-tl from-slate-600 to-slate-300">
                    <span className="block bg-(--bg-color) leading-none p-1.5 rounded-sm text-center text-xs text-slate-700 dark:text-slate-400 whitespace-nowrap">
                        {children}
                    </span>
                </span>
        case 2:
            return <span
                className="p-px inline-block rounded-sm bg-linear-to-tl from-blue-700 to-cyan-500">
                    <span className="block bg-(--bg-color) leading-none p-1.5 rounded-sm text-center text-xs text-sky-700 dark:text-sky-400 whitespace-nowrap">
                        {children}
                     </span>
                </span>
        case 3:
            return <span
                className="p-px inline-block rounded-sm bg-linear-to-tl from-red-600 to-orange-600">
                    <span className="block bg-(--bg-color) leading-none p-1.5 rounded-sm text-center text-xs text-orange-700 dark:text-orange-400 whitespace-nowrap">
                        {children}
                    </span>
                </span>
        case 4:
            return <span
                className="p-px inline-block rounded-sm bg-linear-to-tl from-green-500 to-teal-500">
                    <span className="block bg-(--bg-color) leading-none p-1.5 rounded-sm text-center text-xs text-emerald-700 dark:text-emerald-400 whitespace-nowrap">
                        {children}
                    </span>
                </span>
        case 5:
            return <span
                className="p-px inline-block rounded-sm bg-linear-to-tl from-gray-400 to-gray-100">
                    <span className="block bg-(--bg-color) leading-none p-1.5 rounded-sm text-center text-xs text-gray-700 dark:text-gray-400 whitespace-nowrap">
                        {children}
                    </span>
                </span>
        case 6:
            return <span
                className="p-px inline-block rounded-sm bg-linear-to-tl from-zinc-800 to-zinc-700">
                    <span className="block bg-(--bg-color) leading-none p-1.5 rounded-sm text-center text-xs text-zinc-700 dark:text-zinc-400 whitespace-nowrap">
                        {children}
                    </span>
                </span>
        default:
            return <span
                className="p-px inline-block rounded-sm bg-linear-to-tl from-green-500 to-teal-500">
                    <span className="block bg-(--bg-color) leading-none p-1.5 rounded-sm text-center text-xs text-emerald-700 dark:text-emerald-400 whitespace-nowrap">
                        {children}
                    </span>
                </span>
    }
}
