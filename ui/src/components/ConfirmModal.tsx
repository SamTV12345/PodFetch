import {useAppSelector} from "../store/hooks"
import {CustomButtonPrimary} from "./CustomButtonPrimary"
import {CustomButtonSecondary} from "./CustomButtonSecondary"
import {Modal} from "./Modal"

export type ConfirmModalProps = {
    headerText: string,
    onAccept: ()=>void,
    onReject: ()=>void,
    acceptText: string,
    rejectText: string,
    bodyText: string
}

export const ConfirmModal = ()=>{
    const confirmModalData = useAppSelector(state=>state.common.confirmModalData)

    return (
        <Modal acceptText={confirmModalData?.acceptText} headerText={confirmModalData?.headerText} onAccept={()=>{}} cancelText={confirmModalData?.rejectText} onCancel={()=>{}} onDelete={()=>{}}>
            <div className="mb-4">
                {confirmModalData?.bodyText}
            </div>
            <div className="text-right">
                <CustomButtonSecondary className="border-transparent shadow-none hover:shadow-none text-base text-stone-900 hover:text-stone-600" onClick={confirmModalData?.onReject}>{confirmModalData?.rejectText}</CustomButtonSecondary>
                <CustomButtonPrimary className="bg-red-700 hover:bg-red-600 hover:shadow-red-600" onClick={confirmModalData?.onAccept}>{confirmModalData?.acceptText}</CustomButtonPrimary>
            </div>
        </Modal>
    )
}
