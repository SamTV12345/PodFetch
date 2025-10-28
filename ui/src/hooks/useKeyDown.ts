import { useCallback, useEffect } from 'react'

export const useKeyDown = (
	callback: (evt: KeyboardEvent) => void,
	keys: string[],
	triggerOnInputField: boolean = true,
) => {
	const onKeyDown = useCallback(
		(event: KeyboardEvent) => {
			const target = event.target as HTMLElement | null
			if (!triggerOnInputField && target?.tagName.toLowerCase() === 'input') {
				return
			}

			const wasAnyKeyPressed = keys.some((key: string) => event.key === key)
			if (wasAnyKeyPressed) {
				event.preventDefault()
				callback(event)
			}
		},
		[callback, triggerOnInputField, ...keys, keys.some],
	)
	useEffect(() => {
		document.addEventListener('keydown', onKeyDown)
		return () => {
			document.removeEventListener('keydown', onKeyDown)
		}
	}, [onKeyDown])
}

export const useCtrlPressed = (callback: () => void, keys: string[]) => {
	const onKeyDown = useCallback(
		(event: KeyboardEvent) => {
			const wasAnyKeyPressed = keys.some(
				(key: string) => event.key === key && event.ctrlKey,
			)
			if (wasAnyKeyPressed) {
				event.preventDefault()
				callback()
			}
		},
		[callback, ...keys, keys.some],
	)
	useEffect(() => {
		document.addEventListener('keydown', onKeyDown)
		return () => {
			document.removeEventListener('keydown', onKeyDown)
		}
	}, [onKeyDown])
}
