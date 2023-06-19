import {FC} from "react"
import {createPortal} from "react-dom"
import {useTranslation} from "react-i18next"
import {useAppDispatch, useAppSelector} from "../store/hooks"
import {setModalOpen} from "../store/ModalSlice"
import { Heading2 } from './Heading2'

export interface ModalProps {
    children: any,
    headerText: string|undefined,
    onCancel: () => void,
    onAccept: () => void,
    onDelete: ()=>void
    cancelText:string|undefined,
    acceptText:string|undefined
}

export const Modal:FC<ModalProps>  = ({headerText,children})=>{
    const openModal  = useAppSelector(state=>state.modal.openModal)
    const dispatch = useAppDispatch()
    const {t} = useTranslation()

    return  openModal ? createPortal(<div id="defaultModal" tabIndex={-1} aria-hidden="true" onClick={()=>dispatch(setModalOpen(false))}
    className="fixed inset-0 grid place-items-center bg-[rgba(0,0,0,0.5)] backdrop-blur h-screen overflow-x-hidden overflow-y-auto z-20">

        <div className="relative bg-white max-w-lg p-8 rounded-2xl shadow-[0_4px_16px_rgba(0,0,0,0.2)] w-full" onClick={(e)=>e.stopPropagation()}>
            <button type="button" className="absolute top-4 right-4 bg-transparent" data-modal-toggle="defaultModal" onClick={()=>dispatch(setModalOpen(false))}>
                <span className="material-symbols-outlined text-stone-400 hover:text-stone-600">close</span>
                <span className="sr-only">Close modal</span>
            </button>

            <Heading2 className="mb-4">{headerText || ''}</Heading2>

            {children}
        </div>
    </div>, document.getElementById('modal') as Element):<div></div>
}
