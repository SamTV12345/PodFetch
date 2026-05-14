import { FC } from 'react'
import { Checkbox as BaseCheckbox } from '@base-ui/react/checkbox'
import { Check } from 'lucide-react'

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
                <Check size={18} className="ui-text-inverse" />
            </BaseCheckbox.Indicator>
        </BaseCheckbox.Root>
    )
}
