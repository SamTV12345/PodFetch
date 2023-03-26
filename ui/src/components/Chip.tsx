import {FC, ReactElement} from "react";

type ChipProps = {
    index: number,
    children?: ReactElement|ReactElement[]|string
}
export const Chip:FC<ChipProps> = ({index, children}) => {

    switch (index % 7) {
        case 0:
            return <span
                className="p-2 text-xs rounded-1.8 inline-block whitespace-nowrap text-center bg-gradient-to-tl from-blue-500 to-violet-500 align-baseline font-bold uppercase leading-none text-white">{children}</span>
        case 1:
            return <span
                className="p-2 text-xs rounded-1.8 inline-block whitespace-nowrap text-center bg-gradient-to-tl from-slate-600 to-slate-300 align-baseline font-bold uppercase leading-none text-white">{children}</span>
        case 2:
            return <span
                className="p-2 text-xs rounded-1.8 inline-block whitespace-nowrap text-center bg-gradient-to-tl from-blue-700 to-cyan-500 align-baseline font-bold uppercase leading-none text-white">{children}</span>
        case 3:
            return <span
                className="p-2 text-xs rounded-1.8 inline-block whitespace-nowrap text-center bg-gradient-to-tl from-red-600 to-orange-600 align-baseline font-bold uppercase leading-none text-white">{children}</span>
        case 4:
            return <span
                className="p-2 text-xs rounded-1.8 inline-block whitespace-nowrap text-center bg-gradient-to-tl from-green-500 to-teal-500 align-baseline font-bold uppercase leading-none text-white">{children}</span>
        case 5:
            return <span
                className="p-2 text-xs rounded-1.8 inline-block whitespace-nowrap text-center bg-gradient-to-tl from-gray-400 to-gray-100 align-baseline font-bold uppercase leading-none text-slate-500">{children}</span>
        case 6:
            return <span
                className="p-2 text-xs rounded-1.8 inline-block whitespace-nowrap text-center bg-gradient-to-tl from-zinc-800 to-zinc-700 align-baseline font-bold uppercase leading-none text-white">{children}</span>
        default:
            return <span
                className="p-2 text-xs rounded-1.8 inline-block whitespace-nowrap text-center bg-gradient-to-tl from-green-500 to-teal-500 align-baseline font-bold uppercase leading-none text-white">{children}</span>
    }
}
