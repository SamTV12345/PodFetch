import i18n from "../language/i18n";
import {useState} from "react";

export const Dropdown = ()=>{
    const [language, setLanguage] = useState<string>(i18n.language)

    return <><select id="countries" className="border text-sm rounded-lg
    block p-2.5 bg-gray-700 border-gray-600
    placeholder-gray-400 text-white focus:ring-blue-500 focus:border-blue-500"
    onChange={(v)=>{setLanguage(v.target.value); i18n.changeLanguage(v.target.value)}}
    value={language}>
    <option value="de-DE">Deutsch</option>
        <option value="en">English</option>
        <option value="fr">Français</option>
    </select></>
}
