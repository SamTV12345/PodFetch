import { useEffect } from 'react'

export const useKeyDown = (
	callback: any,
	keys: string[],
	triggerOnInputField: boolean = true,
) => {
	const onKeyDown = (event: KeyboardEvent) => {
		if (
			!triggerOnInputField &&
			(event.target as HTMLElement).tagName === 'INPUT'
		) {
			return
		}
		const wasAnyKeyPressed = keys.some((key: string) => event.key === key)
		if (wasAnyKeyPressed) {
			event.preventDefault()
			callback()
		}
	}
	useEffect(() => {
		document.addEventListener('keydown', onKeyDown)
		return () => {
			document.removeEventListener('keydown', onKeyDown)
		}
	}, [onKeyDown])
}

export const useCtrlPressed = (callback: any, keys: string[]) => {
	const onKeyDown = (event: KeyboardEvent) => {
		const wasAnyKeyPressed = keys.some(
			(key: string) => event.key === key && event.ctrlKey,
		)
		if (wasAnyKeyPressed) {
			event.preventDefault()
			callback()
		}
	}
	useEffect(() => {
		document.addEventListener('keydown', onKeyDown)
		return () => {
			document.removeEventListener('keydown', onKeyDown)
		}
	}, [onKeyDown])
}
