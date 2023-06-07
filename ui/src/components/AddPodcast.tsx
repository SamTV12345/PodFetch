import {useState} from "react"
import {useTranslation} from "react-i18next"
import {useAppSelector} from "../store/hooks"
import {AddHeader} from "./AddHeader"
import {AddTypes} from "../models/AddTypes"
import {FeedURLComponent} from "./FeedURLComponent"
import {Modal} from "./Modal"
import {OpmlAdd} from "./OpmlAdd"
import {ProviderImportComponent} from "./ProviderImportComponent"

export const AddPodcast = ()=>{
    const {t} = useTranslation()
    const [selectedSearchType, setSelectedSearchType] = useState<AddTypes>(AddTypes.ITUNES)
    const configModel = useAppSelector(state=>state.common.configModel)

    return <Modal onCancel={()=>{}} onAccept={()=>{}} headerText={t('add-podcast')!} onDelete={()=>{}}  cancelText={"Abbrechen"} acceptText={"Hinzufügen"} >
        <AddHeader selectedSearchType={selectedSearchType} setSelectedSearchType={setSelectedSearchType} configModel={configModel}/>

        {selectedSearchType!==AddTypes.OPML&& selectedSearchType!==AddTypes.FEED&&
            <ProviderImportComponent selectedSearchType={selectedSearchType}/>
        }
        {
            selectedSearchType===AddTypes.OPML&&<OpmlAdd selectedSearchType={selectedSearchType}/>
        }
        {
            selectedSearchType === AddTypes.FEED&&<FeedURLComponent/>
        }
    </Modal>
}
