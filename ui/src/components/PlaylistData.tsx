import {CustomInput} from "./CustomInput";
import {useTranslation} from "react-i18next";

type PlaylistDataProps = {
    name: string
    onNameChange: (name: string) => void
}

export const PlaylistData = ({name, onNameChange}: PlaylistDataProps)=>{
    const {t} = useTranslation()


    return <div className="mb-6 rounded-xl border ui-border p-4">
        <div className="mb-3 text-sm ui-text-muted">
            Gib deiner Playlist einen Namen, damit du sie später schnell wiederfindest.
        </div>
        <div className="grid grid-cols-1 xs:grid-cols-[1fr_auto] items-center gap-2 xs:gap-6">
        <fieldset className="xs:contents mb-4">
            <label className="ml-2 text-sm ui-text-muted" htmlFor="playlist-name">{t('playlist-name')}</label>

            <div className="flex flex-col gap-2">
                <div className="flex">
                            <CustomInput id="playlist-name" className="border-gray-500 border-2" onChange={e=>onNameChange(e.target.value)} value ={name} />

                </div>
            </div>
        </fieldset>
    </div></div>
}
