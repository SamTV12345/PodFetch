import { FC } from 'react'
import { Checkbox as BaseCheckbox } from '@base-ui/react/checkbox'
import 'material-symbols/outlined.css'

type CustomCheckboxProps = {
    className?: string,
    id?: string,
    name?: string,
    onChange?: (checked: boolean) => void,
    value?: boolean
}

export const CustomCheckbox: FC<CustomCheckboxProps> = ({ className = '', id, name, onChange, value }) => {
    return (
        <BaseCheckbox.Root
            checked={value}
            onCheckedChange={onChange}
            className={`align-middle ui-input-surface data-[checked]:ui-bg-accent h-6 w-6 rounded-sm ${className}`}
            id={id}
            name={name}
        >
            <BaseCheckbox.Indicator>
                <span className="material-symbols-outlined ui-text-inverse">check</span>
            </BaseCheckbox.Indicator>
        </BaseCheckbox.Root>
    )
}
