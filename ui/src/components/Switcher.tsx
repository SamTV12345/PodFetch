import { FC } from 'react'

type SwitcherProps = {
    checked: boolean,
    className?: string,
    id?: string,
    setChecked: (checked: boolean) => void
}

export const Switcher: FC<SwitcherProps> = ({ checked, className = '', id, setChecked }) => {
    return (
        <div className={`relative inline-flex items-center cursor-pointer ${className}`} onClick={() => {
            setChecked(!checked)
        }}>
            <input checked={checked} className="sr-only peer" id={id} onChange={() => {
            setChecked(!checked)
        }} type="checkbox" value="" />

            <div className={
                /* Container */
                "peer relative h-6 w-11 rounded-full bg-[--border-color] peer-checked:bg-[--accent-color] " +
                /* Switch */
                "after:absolute after:top-[2px] after:left-[2px] after:h-5 after:w-5 after:bg-[--bg-color] after:content-[''] after:rounded-full after:shadow-[0_2px_4px_rgba(0,0,0,var(--shadow-opacity))] hover:after:shadow-[0_2px_8px_rgba(0,0,0,var(--shadow-opacity-hover))] after:transition-all peer-checked:after:translate-x-full"
            }></div>
        </div>
    )
}
