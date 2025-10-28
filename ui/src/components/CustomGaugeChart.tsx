import {
	ArcElement,
	type ChartArea,
	type ChartData,
	Chart as ChartJS,
	type ChartOptions,
	Legend,
	Tooltip,
	type TooltipItem,
} from 'chart.js'
import { type FC, useEffect, useRef, useState } from 'react'
import { Doughnut } from 'react-chartjs-2'

interface CustomGaugeChartProps {
	fill?: string | string[]
	labels?: string[]
	labelUnit?: 'capacity' | 'percent'
	max: number
	value: number
}

function createGradient(
	ctx: CanvasRenderingContext2D,
	area: ChartArea,
	fill: string[],
) {
	const gradient = ctx.createLinearGradient(area.left, 0, area.right, 0)
	if (!fill[0] || !fill[1]) return gradient

	gradient.addColorStop(0, fill[0])
	gradient.addColorStop(1, fill[1])

	return gradient
}

export const CustomGaugeChart: FC<CustomGaugeChartProps> = ({
	fill,
	labels,
	labelUnit,
	max,
	value,
}) => {
	const chartRef = useRef<ChartJS<'doughnut'>>(null)
	const [chartData, setChartData] = useState<ChartData<'doughnut'>>({
		datasets: [],
	})
	const [chartOptions, setChartOptions] = useState<ChartOptions<'doughnut'>>()

	const gigaByte = 10 ** 9
	const teraByte = 10 ** 12

	useEffect(() => {
		const chart = chartRef.current

		if (!chart) return

		/* Get CSS variables */
		const style = getComputedStyle(document.body)
		const bgColor = style.getPropertyValue('--bg-color')
		const sliderBgColor = style.getPropertyValue('--slider-bg-color')

		const chartData = {
			labels,
			datasets: [
				{
					data: [value, max - value],
					backgroundColor: [
						Array.isArray(fill)
							? createGradient(chart.ctx, chart.chartArea, fill)
							: fill,
						sliderBgColor,
					],
					borderColor: bgColor,
					borderRadius: [
						{ innerStart: 16, outerStart: 16, innerEnd: 0, outerEnd: 0 },
						{ innerStart: 0, outerStart: 0, innerEnd: 16, outerEnd: 16 },
					],
				},
			],
		}

		const chartOptions = {
			circumference: 240,
			cutout: '75%',
			maintainAspectRatio: false,
			plugins: {
				legend: {
					display: false,
				},
				tooltip: {
					callbacks: {
						label: (context: TooltipItem<'doughnut'>) => {
							let label = context.dataset.label || ''

							if (label) {
								label += ': '
							}

							if (labelUnit === 'capacity') {
								if (context.parsed > teraByte) {
									return (
										label +
										': ' +
										(context.parsed / teraByte).toFixed(2) +
										' TB'
									)
								} else {
									return (
										label +
										': ' +
										(context.parsed / gigaByte).toFixed(2) +
										' GB'
									)
								}
							} else if (labelUnit === 'percent') {
								return `${label}: ${context.parsed.toFixed(2)} %`
							} else {
								return label
							}
						},
					},
				},
			},
			responsive: true,
			rotation: 240,
		}

		setChartData(chartData)
		setChartOptions(chartOptions)
	}, [fill, labelUnit, labels, max, value])

	ChartJS.register(ArcElement, Tooltip, Legend)

	return (
		<div className="relative w-11/12 mx-auto">
			<Doughnut data={chartData} options={chartOptions} ref={chartRef} />

			<span className="flex items-center justify-center absolute inset-0 pl-2 pt-6 text-4xl xs:text-3xl sm:text-4xl md:text-3xl text-(--fg-color) -z-10">
				{Math.round((value / max) * 100)}%
			</span>
		</div>
	)
}
