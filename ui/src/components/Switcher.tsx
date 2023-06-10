import {FC} from "react"

type SwitcherProps = {
    checked: boolean,
    setChecked: (checked: boolean) => void
}

export const Switcher:FC<SwitcherProps> = ({checked,setChecked}) => {
    return <label className="relative inline-flex items-center cursor-pointer">
        <input type="checkbox" value="" className="sr-only peer" checked={checked} onChange={()=>{
            setChecked(!checked)
        }}/>

        <div className={
            /* Container */
            "peer h-6 w-11 rounded-full bg-stone-200 peer-checked:bg-mustard-600 " +
            /* Switch */
            "after:absolute after:top-[2px] after:left-[2px] after:h-5 after:w-5 after:bg-white after:content-[''] after:rounded-full after:shadow-[0_2px_4px_rgba(0,0,0,0.2)] hover:after:shadow-[0_2px_8px_rgba(0,0,0,0.5)] after:transition-all peer-checked:after:translate-x-full"
        }></div>
    </label>
}
