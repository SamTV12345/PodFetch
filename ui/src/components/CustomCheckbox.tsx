import {FC} from "react"
import * as Checkbox from "@radix-ui/react-checkbox"
import "material-symbols/outlined.css"

type CustomCheckboxProps = {
    className?: string,
    id?: string,
    name?: string,
    onChange?: any,
    value?: Checkbox.CheckedState,
}

export const CustomCheckbox:FC<CustomCheckboxProps> = ({className = '', id, name, onChange, value}) => {
    return (
        <Checkbox.Root checked={value} className={`align-middle bg-stone-200 data-[state=checked]:bg-mustard-600 h-6 w-6 rounded ${className}`} id={id} onCheckedChange={onChange} name={name}>
            <Checkbox.Indicator>
                <span className="material-symbols-outlined text-white">check</span>
            </Checkbox.Indicator>
        </Checkbox.Root>    
    )
}
