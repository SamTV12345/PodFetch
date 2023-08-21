import {CustomInput} from "./CustomInput";
import {useTranslation} from "react-i18next";
import {useAppDispatch, useAppSelector} from "../store/hooks";
import {setCurrentPlaylistToEdit} from "../store/PlaylistSlice";




export const PlaylistData = ()=>{
    const {t} = useTranslation()
    const currentPlaylistToEdit = useAppSelector(state=>state.playlist.currentPlaylistToEdit)
    const dispatch = useAppDispatch()

    const changeName = (e:string)=>{
        dispatch(setCurrentPlaylistToEdit({
            name: e,
            id: currentPlaylistToEdit!.id,
            items: currentPlaylistToEdit!.items
        }))
    }


    return <div className="grid grid-cols-1 xs:grid-cols-[1fr_auto] items-center gap-2 xs:gap-6 mb-10">
        <fieldset className="xs:contents mb-4">
            <label className="ml-2 text-sm text-stone-600" htmlFor="use-existing-filenames">{t('playlist-name')}</label>

            <div className="flex flex-col gap-2">
                <div className="flex">
                            <CustomInput id="use-existing-filenames" className="border-gray-500 border-2" onChange={e=>changeName(e.target.value)} value ={currentPlaylistToEdit?.name} />

                </div>
            </div>
        </fieldset>
    </div>
}
