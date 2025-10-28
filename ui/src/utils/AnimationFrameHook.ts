import { type EffectCallback, useCallback, useEffect, useRef } from 'react'

export const useAnimationFrame = (
	callback: EffectCallback,
	wait = 0,
): ((...args: Parameters<EffectCallback>) => void) => {
	const rafId = useRef(0)
	const render = useCallback(
		(...args: Parameters<EffectCallback>) => {
			cancelAnimationFrame(rafId.current)
			const timeStart = performance.now()

			const renderFrame = (timeNow: number) => {
				if (timeNow - timeStart < wait) {
					rafId.current = requestAnimationFrame(renderFrame)
					return
				}
				callback(...args)
			}
			rafId.current = requestAnimationFrame(renderFrame)
		},
		[callback, wait],
	)

	useEffect(() => cancelAnimationFrame(rafId.current), [])
	return render
}
