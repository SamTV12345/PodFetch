import { createPortal } from 'react-dom'
import { Heading2 } from './Heading2'
import 'material-symbols/outlined.css'
import {FC, ReactNode} from "react";

type InfoModalProps = {
    open: boolean,
    setOpen: (open: boolean) => void,
    children: ReactNode|ReactNode[],
    heading: string
}


export const EpisodeFormatModal:FC<InfoModalProps> = ({open,setOpen, children, heading}) => {

    return createPortal(
        <div
            id="defaultModal"
            tabIndex={-1}
            aria-hidden="true"
            onClick={() => setOpen(false)}
            className={`fixed inset-0 grid place-items-center bg-[rgba(0,0,0,0.5)] backdrop-blur overflow-y-auto overflow-x-hidden transition-opacity z-30
            ${!open && 'pointer-events-none'}
            ${open ? 'opacity-100' : 'opacity-0'}`}
        >
            <div className={`relative ui-surface max-w-2xl p-8 rounded-2xl shadow-[0_4px_16px_rgba(0,0,0,var(--shadow-opacity))] ${open ? 'opacity-100' : 'opacity-0'}`} onClick={e => e.stopPropagation()}>
                <button
                    type="button"
                    onClick={() => setOpen(false)}
                    className="absolute top-4 right-4 bg-transparent"
                    data-modal-hide="defaultModal">
                    <span className="material-symbols-outlined ui-modal-close hover:ui-modal-close-hover">close</span>
                    <span className="sr-only">Close modal</span>
                </button>

                <div className="mb-4">
                    <Heading2 className="inline align-middle mr-2">{heading}</Heading2>
                </div>

                <div className="w-96">
                    {children}
                </div>
            </div>
        </div>, document.getElementById('modal1')!
    )
}
