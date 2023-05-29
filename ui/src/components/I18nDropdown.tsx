import i18n from "../language/i18n";
import {useState} from "react";
import {LanguageIcon} from "../icons/LanguageIcon";
export const Dropdown = ()=>{
    const [language, setLanguage] = useState<string>(i18n.language)

    return <div className="flex ">
        <LanguageIcon/><select id="countries" className=" text-sm rounded-lg
    block
    placeholder-gray-400 text-black"
    onChange={(v)=>{setLanguage(v.target.value); i18n.changeLanguage(v.target.value)}}
    value={language}>
    <option value="de-DE">Deutsch</option>
        <option value="en">English</option>
        <option value="fr">Français</option>
    </select>
    </div>
}
