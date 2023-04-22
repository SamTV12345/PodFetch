import {useState} from "react";
import {useTranslation} from "react-i18next";
import axios from "axios";
import {apiURL} from "../utils/Utilities";

export const OPMLExport = () => {
    const [exportType, setExportType] = useState<string>('local')
    const {t} = useTranslation()

    const downloadOPML = ()=>{
        axios({
            url: apiURL+"/settings/opml/"+exportType, //your url
            method: 'GET',
            responseType: "blob"
        }).then((response) => {
            // create file link in browser's memory
            const href = URL.createObjectURL(response.data);

            // create "a" HTML element with href to file & click
            const link = document.createElement('a');
            link.href = href;
            link.setAttribute('download', 'podcast_'+exportType+".opml"); //or any other extension
            document.body.appendChild(link);
            link.click();

            // clean up "a" element & remove ObjectURL
            document.body.removeChild(link);
            URL.revokeObjectURL(href);
        });
    }

    return                 <div className="bg-slate-900 rounded p-5 text-white">
        <h1 className="text-2xl text-center">{t('opml-export')}</h1>
        <div className="mt-2 flex">
            <div className="mt-2 mr-2">{t('i-want-to-export')}</div>
            <select id="countries" className="border  text-sm rounded-lg
                              block p-2.5 bg-gray-700 border-gray-600
                            placeholder-gray-400 text-white focus:ring-blue-500 focus:border-blue-500"
                    onChange={(v)=>{setExportType(v.target.value)}}
                    value={exportType}>
                <option value="local">{t('local')}</option>
                <option value="online">{t('online')}</option>
            </select>
            <div className="mt-2 ml-2">{t('export')}</div>
        </div>
        <div className="flex">
            <div className="flex-1"></div>
            <button className="bg-blue-600 rounded p-2" onClick={()=>downloadOPML()}>{t('download')}</button>
        </div>
    </div>
}
