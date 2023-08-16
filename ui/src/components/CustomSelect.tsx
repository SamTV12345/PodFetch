import { FC } from 'react'
import { useTranslation } from 'react-i18next'
import { TFunction } from 'i18next'
import * as Select from '@radix-ui/react-select'
import 'material-symbols/outlined.css'

export type Option = {
    label?: string,
    translationKey?: string,
    value: string
}

type CustomSelectProps = {
    className?: string,
    defaultValue?: string,
    iconName?: string,
    id?: string,
    name?: string,
    onChange?: (v: string) => void,
    options: Array<Option>,
    placeholder?: string | TFunction,
    value: string
}

export const CustomSelect: FC<CustomSelectProps> = ({ className = '', defaultValue, iconName, id, name, onChange, options, placeholder, value }) => {
    const {t} = useTranslation()

    return (
        <Select.Root defaultValue={defaultValue} name={name} onValueChange={onChange} value={value}>
            <Select.Trigger className={`flex items-center border border-[--border-color] pl-6 pr-2 py-2 rounded-full text-sm text-[--select-text-color] ${className}`} id={id}>
                {iconName &&
                    <span className="icon material-symbols-outlined align-middle !leading-[1.25rem] -ml-2 mr-1 text-[--select-icon-color]">{iconName}</span>
                }

                <span className="value grow">
                    <Select.Value placeholder={placeholder as string}/>
                </span>

                <Select.Icon>
                    <span className="expand-icon material-symbols-outlined align-middle !leading-[1.25rem] ml-1 text-[--select-icon-color]">expand_more</span>
                </Select.Icon>
            </Select.Trigger>

            <Select.Portal>
                <Select.Content className="overflow-hidden bg-[--bg-color] rounded-lg shadow-[0_4px_16px_rgba(0,0,0,var(--shadow-opacity))] z-30">
                    <Select.ScrollUpButton />

                    <Select.Viewport className="p-2">
                        {options.map((option) =>
                            <Select.Item className="relative pl-6 pr-4 py-1.5 rounded text-sm text-[--select-text-color] hover:bg-[--accent-color] hover:text-[--bg-color]" key={option.value} value={option.value}>
                                <Select.ItemIndicator className="absolute left-0">
                                    <span className="material-symbols-outlined align-middle !leading-none !text-xl">check</span>
                                </Select.ItemIndicator>

                                <Select.ItemText>{option.translationKey ? t(option.translationKey) : option.label}</Select.ItemText>
                            </Select.Item>
                        )}
                    </Select.Viewport>

                    <Select.ScrollDownButton />
                    <Select.Arrow />
                </Select.Content>
            </Select.Portal>
        </Select.Root>
    )
}
