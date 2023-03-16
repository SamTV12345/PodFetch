import i18n from "../language/i18n";
import {useState} from "react";

export const Dropdown = ()=>{
    const [language, setLanguage] = useState<string>(i18n.language)

    return <><select id="countries" className="bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg
    focus:ring-blue-500 focus:border-blue-500 block p-2.5 dark:bg-gray-700 dark:border-gray-600
    dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500"
    onChange={(v)=>{setLanguage(v.target.value); i18n.changeLanguage(v.target.value)}}
    value={language}>
    <option value="de-DE">Deutsch</option>
        <option value="en">English</option>
        </select></>
}
