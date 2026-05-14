import { FC, useMemo } from 'react'
import { useTranslation } from 'react-i18next'
import { TFunction } from 'i18next'
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from '@/components/ui/select'
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
    value: string,
    disabled?: boolean
}

export const CustomSelect: FC<CustomSelectProps> = ({ className = '', defaultValue, iconName, id, name, onChange, options, placeholder, value, disabled }) => {
    const { t } = useTranslation()

    // Base UI's `SelectValue` renders the LABEL of the currently selected
    // item by looking it up in `items` on the root. Without this map the
    // trigger falls back to the raw `value` string (e.g. "de-DE" or
    // "TITLE"), bypassing i18n. Memoised so changing language re-translates.
    const itemsMap = useMemo(() => {
        const out: Record<string, string> = {}
        for (const opt of options) {
            out[opt.value] = opt.translationKey ? t(opt.translationKey) : (opt.label ?? opt.value)
        }
        return out
    }, [options, t])

    return (
        <Select
            disabled={disabled}
            defaultValue={defaultValue}
            name={name}
            onValueChange={(v) => onChange?.(v ?? '')}
            value={value}
            items={itemsMap}
        >
            <SelectTrigger
                id={id}
                // bg-transparent / dark:bg-transparent override shadcn's
                // default `dark:bg-input/30` fill, which gave the trigger a
                // gray wash in dark mode and made it blend into the surface.
                className={`flex items-center border ui-border bg-transparent dark:bg-transparent pl-6 pr-2 py-2 rounded-full text-sm text-(--select-text-color) ${className}`}
            >
                {iconName && (
                    <span className="icon material-symbols-outlined align-middle leading-[1.25rem]! -ml-2 mr-1 text-(--select-icon-color)">{iconName}</span>
                )}
                <span className="value grow">
                    <SelectValue placeholder={placeholder as string} />
                </span>
            </SelectTrigger>

            <SelectContent className="overflow-hidden ui-surface rounded-lg shadow-[0_4px_16px_rgba(0,0,0,var(--shadow-opacity))]">
                {options.map((option) => (
                    <SelectItem
                        key={option.value}
                        value={option.value}
                        className="relative pl-6 pr-4 py-1.5 rounded-sm text-sm text-(--select-text-color)"
                    >
                        {option.translationKey ? t(option.translationKey) : option.label}
                    </SelectItem>
                ))}
            </SelectContent>
        </Select>
    )
}
