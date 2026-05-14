import {FC, useId, useMemo} from 'react'
import {Cell, Pie, PieChart, Tooltip} from 'recharts'

interface CustomGaugeChartProps {
    fill?: string | string[]
    labels?: string[]
    labelUnit?: 'capacity' | 'percent'
    max: number
    value: number
}

const GIGABYTE = 1e9
const TERABYTE = 1e12

function formatLabel(value: number, unit?: 'capacity' | 'percent') {
    if (unit === 'capacity') {
        return value > TERABYTE
            ? `${(value / TERABYTE).toFixed(2)} TB`
            : `${(value / GIGABYTE).toFixed(2)} GB`
    }
    if (unit === 'percent') {
        return `${value.toFixed(2)} %`
    }
    return String(value)
}

/**
 * 240° doughnut gauge that mirrors the chart.js implementation it
 * replaces. Recharts' `PieChart` with `startAngle`/`endAngle` matches
 * the original cutout-75% arc closely; a linear-gradient `<defs>` block
 * keeps the two-stop fill option from the old API.
 *
 * Angle convention: Recharts measures from 3 o'clock counter-clockwise,
 * so a 240° sweep starting at 210° and ending at -30° produces a gauge
 * opening at the bottom.
 */
export const CustomGaugeChart: FC<CustomGaugeChartProps> = ({fill, labels, labelUnit, max, value}) => {
    const gradientId = useId()
    const sliderBg = 'var(--slider-bg-color)'
    const data = useMemo(() => ([
        {name: labels?.[0] ?? 'used', value},
        {name: labels?.[1] ?? 'free', value: Math.max(0, max - value)},
    ]), [labels, value, max])

    const usedFill = Array.isArray(fill) ? `url(#${gradientId})` : (fill ?? 'var(--accent-color)')
    const percent = max > 0 ? Math.round((value / max) * 100) : 0

    return (
        // `isolate` scopes the negative z-index so the center label stays
        // behind the SVG hover targets without falling behind <body>'s
        // background (which is what happened with `relative` alone).
        <div className="relative isolate w-11/12 mx-auto aspect-square">
            <PieChart width={280} height={280} className="!w-full !h-full">
                {Array.isArray(fill) && (
                    <defs>
                        <linearGradient id={gradientId} x1="0" y1="0" x2="1" y2="0">
                            <stop offset="0%" stopColor={fill[0]}/>
                            <stop offset="100%" stopColor={fill[1]}/>
                        </linearGradient>
                    </defs>
                )}
                <Pie
                    data={data}
                    dataKey="value"
                    nameKey="name"
                    cx="50%"
                    cy="50%"
                    innerRadius="75%"
                    outerRadius="100%"
                    startAngle={210}
                    endAngle={-30}
                    paddingAngle={0}
                    stroke="var(--bg-color)"
                    cornerRadius={16}
                    isAnimationActive={false}
                >
                    <Cell fill={usedFill}/>
                    <Cell fill={sliderBg}/>
                </Pie>
                {/* Style the tooltip via inline styles - Recharts injects
                    them on the floating wrapper, so Tailwind classes on the
                    component don't reach it. Match the popover / card token
                    set so the box looks like the rest of the app. */}
                <Tooltip
                    contentStyle={{
                        backgroundColor: 'var(--popover)',
                        color: 'var(--popover-foreground)',
                        border: '1px solid var(--border)',
                        borderRadius: '0.5rem',
                        boxShadow: '0 4px 16px rgba(0, 0, 0, var(--shadow-opacity))',
                        padding: '0.5rem 0.75rem',
                    }}
                    itemStyle={{
                        color: 'var(--popover-foreground)',
                        padding: 0,
                    }}
                    labelStyle={{ color: 'var(--popover-foreground)' }}
                    cursor={false}
                    formatter={(v, name) => [formatLabel(Number(v ?? 0), labelUnit), String(name ?? '')]}
                />
            </PieChart>
            {/* `pointer-events-none` keeps the slice tooltips reachable
                even where the label overlaps. -z-10 alone (without parent
                isolation) used to leak the label behind <body>, hence the
                "Prozent ist verschwunden" report. */}
            <span className="pointer-events-none flex items-center justify-center absolute inset-0 pl-2 pt-6 text-4xl xs:text-3xl sm:text-4xl md:text-3xl ui-text -z-10">
                {percent}%
            </span>
        </div>
    )
}
