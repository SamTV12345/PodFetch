import { FC } from 'react'
import * as Checkbox from '@radix-ui/react-checkbox'
import 'material-symbols/outlined.css'

type CustomCheckboxProps = {
    className?: string,
    id?: string,
    name?: string,
    onChange?: (checked: Checkbox.CheckedState)=>void,
    value?: Checkbox.CheckedState
}

export const CustomCheckbox: FC<CustomCheckboxProps> = ({ className = '', id, name, onChange, value }) => {
    return (
        <Checkbox.Root checked={value} className={`align-middle ui-input-surface data-[state=checked]:ui-bg-accent h-6 w-6 rounded-sm ${className}`} id={id} onCheckedChange={onChange} name={name}>
            <Checkbox.Indicator>
                <span className="material-symbols-outlined ui-text-inverse">check</span>
            </Checkbox.Indicator>
        </Checkbox.Root>    
    )
}
