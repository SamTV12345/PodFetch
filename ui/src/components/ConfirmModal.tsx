import {Modal} from "./Modal";
import {useAppSelector} from "../store/hooks";

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

    return <Modal acceptText={confirmModalData?.acceptText} headerText={confirmModalData?.headerText} onAccept={()=>{}} cancelText={confirmModalData?.rejectText}
                  onCancel={()=>{}} onDelete={()=>{}}>
        <div>
            {confirmModalData?.bodyText}
        </div>
        <div className="grid grid-cols-2 gap-5 w-2/4">
            <button className="bg-green-800 p-2 rounded active:scale-95 hover:bg-green-700" onClick={confirmModalData?.onReject}>{confirmModalData?.rejectText}</button>
            <button className="bg-red-700 p-2 rounded active:scale-95 hover:bg-red-600" onClick={confirmModalData?.onAccept}>{confirmModalData?.acceptText}</button>
        </div>
    </Modal>
}
