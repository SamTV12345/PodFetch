import {FC} from "react";
import {useTranslation} from "react-i18next";
import {ConfigModel} from "../models/SysInfo";
import {AddTypes} from "../models/AddTypes";

type AddHeaderProps = {
    selectedSearchType: AddTypes;
    setSelectedSearchType: (type: AddTypes) => void,
    configModel: ConfigModel|undefined
}

export const AddHeader:FC<AddHeaderProps> = ({selectedSearchType,setSelectedSearchType, configModel})=>{
    const {t} = useTranslation()
    return             <ul id="podcast-add-decider" className="flex flex-wrap text-sm font-medium text-center border-b border-gray-700 text-gray-400">
        <li className="mr-2">
            <div className={`cursor-pointer inline-block p-4 rounded-t-lg ${selectedSearchType=== "itunes"&& 'active'}`} onClick={()=>setSelectedSearchType(AddTypes.ITUNES)}>iTunes</div>
        </li>
        {configModel?.podindexConfigured&&<li className="mr-2">
            <div
                className={`cursor-pointer inline-block p-4 rounded-t-lg hover:bg-gray-800 hover:text-gray-300 ${selectedSearchType === "podindex" && 'active'}`} onClick={()=>setSelectedSearchType(AddTypes.PODINDEX)}>PodIndex</div>
        </li>}
        <li>
            <div
                className={`cursor-pointer inline-block p-4 rounded-t-lg hover:bg-gray-800 hover:text-gray-300 ${selectedSearchType === "opml" && 'active'}`} onClick={()=>setSelectedSearchType(AddTypes.OPML)}>{t('opml-file')}</div>
        </li>
        <li>
            <div
                className={`cursor-pointer inline-block p-4 rounded-t-lg hover:bg-gray-800 hover:text-gray-300 ${selectedSearchType === "feed" && 'active'}`} onClick={()=>setSelectedSearchType(AddTypes.FEED)}>{t('rss-feed-url')}</div>
        </li>
    </ul>
}
