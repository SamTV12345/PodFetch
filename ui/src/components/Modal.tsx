import { FC } from 'react'
import { createPortal } from 'react-dom'
import useModal from '../store/ModalSlice'
import { Heading2 } from './Heading2'

export interface ModalProps {
    children: any,
    headerText: string|undefined,
    onCancel: () => void,
    onAccept: () => void,
    onDelete: ()=>void
    cancelText: string|undefined,
    acceptText: string|undefined
}

export const Modal: FC<ModalProps>  = ({ headerText,children }) => {
    const openModal  = useModal(state => state.openModal)
    const setModalOpen = useModal(state => state.setOpenModal)

    return openModal ? createPortal(
        <div id="defaultModal" tabIndex={-1} aria-hidden="true" onClick={() => setModalOpen(false)}
        className="fixed inset-0 grid place-items-center bg-[rgba(0,0,0,0.5)] backdrop-blur h-screen overflow-x-hidden overflow-y-auto z-20">

            <div className="relative bg-[--bg-color] max-w-lg p-8 rounded-2xl shadow-[0_4px_16px_rgba(0,0,0,var(--shadow-opacity))] w-full" onClick={(e) => e.stopPropagation()}>
                <button type="button" className="absolute top-4 right-4 bg-transparent" data-modal-toggle="defaultModal" onClick={() => setModalOpen(false)}>
                    <span className="material-symbols-outlined text-[--modal-close-color] hover:text-[--modal-close-color-hover]">close</span>
                    <span className="sr-only">Close modal</span>
                </button>

                <Heading2 className="mb-4">{headerText || ''}</Heading2>

                {children}
            </div>
        </div>, document.getElementById('modal') as Element
    ):<div></div>
}
