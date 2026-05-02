import { FC } from 'react'
import { useTranslation } from 'react-i18next'
import { Switcher } from './Switcher'

export const SPONSORBLOCK_CATEGORIES = [
    'sponsor',
    'selfpromo',
    'interaction',
    'intro',
    'outro',
    'preview',
    'music_offtopic',
    'filler',
] as const

export type SponsorBlockCategory = (typeof SPONSORBLOCK_CATEGORIES)[number]

type Props = {
    categories: string[]
    onChange: (next: string[]) => void
    loading?: boolean
    disabled?: boolean
}

export const SponsorBlockCategorySettings: FC<Props> = ({
    categories,
    onChange,
    loading,
    disabled,
}) => {
    const { t } = useTranslation()
    const enabled = new Set(categories)

    const toggle = (cat: SponsorBlockCategory) => {
        const next = new Set(enabled)
        if (next.has(cat)) {
            next.delete(cat)
        } else {
            next.add(cat)
        }
        onChange(SPONSORBLOCK_CATEGORIES.filter((c) => next.has(c)))
    }

    return (
        <div
            className={`grid grid-cols-1 xs:grid-cols-[1fr_auto] gap-2 xs:gap-x-6 ${
                disabled ? 'opacity-50 pointer-events-none' : ''
            }`}
        >
            {SPONSORBLOCK_CATEGORIES.map((cat) => (
                <div
                    key={cat}
                    className="flex flex-col gap-2 xs:contents mb-2"
                >
                    <label htmlFor={`sb-cat-${cat}`} className="ui-text">
                        {t(`category-${cat.replace('_', '-')}`)}
                    </label>
                    <Switcher
                        id={`sb-cat-${cat}`}
                        loading={loading}
                        checked={enabled.has(cat)}
                        className="xs:justify-self-end"
                        onChange={() => toggle(cat)}
                    />
                </div>
            ))}
        </div>
    )
}
