import {FC} from "react";

type SwitcherProps = {
    checked: boolean,
    setChecked: (checked: boolean) => void
}

export const Switcher:FC<SwitcherProps> = ({checked,setChecked}) => {
    return <label className="relative inline-flex items-center cursor-pointer">
        <input type="checkbox" value="" className="sr-only peer" checked={checked} onChange={()=>{
            setChecked(!checked)
        }}/>
            <div
                className="w-11 h-6 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-blue-800 rounded-full peer bg-gray-700 peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all border-gray-600 peer-checked:bg-blue-600"></div>
    </label>
}
