import {FC, ReactNode} from "react";

interface SysCardProps {
    title: string;
    children: ReactNode|ReactNode[];
}
// <p className="text-gray-700 dark:text-gray-400">Lorem ipsum dolor sit amet, consectetur adipisicing elit. Quisquam, quae.</p>
export const SysCard:FC<SysCardProps> = ({children,title}) => {
    return <div className=" bg-white border border-gray-200 rounded-lg shadow dark:bg-gray-800 dark:border-gray-700 h-full">
        <div className="p-5">
            <h5 className="mb-2 text-2xl font-bold tracking-tight text-gray-900 dark:text-white">{title}</h5>
            <div>
                {children}
            </div>
        </div>
    </div>
}
