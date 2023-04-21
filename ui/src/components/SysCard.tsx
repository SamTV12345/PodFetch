import {FC, ReactNode} from "react";

interface SysCardProps {
    title: string;
    children: ReactNode|ReactNode[];
}
// <p className="text-gray-700 dark:text-gray-400">Lorem ipsum dolor sit amet, consectetur adipisicing elit. Quisquam, quae.</p>
export const SysCard:FC<SysCardProps> = ({children,title}) => {
    return <div className="border rounded-lg shadow bg-gray-800 border-gray-700 h-full break-words">
        <div className="p-5">
            <h5 className="mb-2 sm:text-xl md:text-2xl font-bold tracking-tight text-white">{title}</h5>
            <div>
                {children}
            </div>
        </div>
    </div>
}
